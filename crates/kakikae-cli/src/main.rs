#![feature(int_roundings)]

mod error;
mod utils;
mod kamakiri2;

use nusb::io::{EndpointRead, EndpointWrite};
use nusb::{Device};
use std::io::{stdout, Read, Write};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use crate::error::KakikaeError;
use crate::kamakiri2::Kamakiri2;

const SHORT_TIMEOUT: Duration = Duration::from_millis(5);
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);
const MTK_VID: u16 = 0x0e8d;
const BROM_PID: u16 = 0x0003;

const PAYLOAD_ADDRESS: u32 = 0x100A00;
const WATCHDOG_PTR: u32 = 0x10007000;

struct DeviceEndpoint {
    in_max_size: usize,
    ep_in: EndpointRead<nusb::transfer::Bulk>,
    ep_out: EndpointWrite<nusb::transfer::Bulk>,
}

#[tokio::main]
async fn main() -> Result<(), KakikaeError> {

    let mut usb_device = find_usb_device().await?;
    let mut dev_ep = connect_usb(&usb_device).await?;

    let handshake_magic: &[u8] = &[0xA0, 0x0A, 0x50, 0x05];
    handshake(&mut dev_ep, handshake_magic).await?;

    let hw_code = cmd_get_hwcode(&mut dev_ep).await?;
    println!("Device hwcode: 0x{hw_code:x}");

    if hw_code != 0x813 {
        return Err(KakikaeError::UnsupportedDevice)
    }

    let stage0 = utils::read_stage_data("/kakikae-s0.bin")?;
    println!("Attempting to load stage 0 via kamakiri2...");

    let mut kamakiri2 = Kamakiri2::new(&mut usb_device, &mut dev_ep);
    kamakiri2.send_da(&stage0).await?;

    stage_cmd(&mut dev_ep, 0x48454C4F, true).await?; // handshake

    let stage1 = utils::read_stage_data("/kakikae-s1.bin")?;
    println!("Sending Stage 1...");
    stage_write_data(&mut dev_ep, &stage1, kakikae_shared::S1_BASE_ADDR as u32).await?;

    println!("Stage 1 Handshake...");
    stage_cmd(&mut dev_ep, 0x48454C4F, true).await?; // Handshake

    println!("Switching to BROM cmd handler...");
    stage_cmd(&mut dev_ep, 0x434f4d44, true).await?; // COMD command

    println!("Booting...");
    cmd_boot_pl(&mut dev_ep).await?;

    println!("Device should start booting in a bit.");

    Ok(())
}
async fn find_usb_device() -> Result<Device, KakikaeError> {

    println!("Waiting for device (USB)...");
    loop {
        for device in nusb::list_devices().await? {
            if device.vendor_id() == MTK_VID && device.product_id() == BROM_PID {
                sleep(DEFAULT_TIMEOUT).await; // Wait a bit to avoid random device open fails
                match device.open().await {
                    Ok(found) => {
                        println!("Found a device in BootROM mode.",);
                        return Ok(found)
                    }
                    Err(e) => println!("Failed to open device: {e}")
                }
            }
        }
        sleep(DEFAULT_TIMEOUT).await;
        stdout().flush()?;
    }
}
async fn connect_usb(usb_device: &Device) -> Result<DeviceEndpoint, KakikaeError> {
    if cfg!(target_os = "linux") {
        usb_device.detach_kernel_driver(1)?;
    }
    usb_device.set_configuration(1).await?;
    let interface = usb_device.claim_interface(1).await?;

    let ep_out = interface.endpoint::<nusb::transfer::Bulk, nusb::transfer::Out>(0x1)?;
    let ep_in = interface.endpoint::<nusb::transfer::Bulk, nusb::transfer::In>(0x81)?;
    let in_max_size = ep_in.max_packet_size();
    let mut ep_read = ep_in.reader(32);
    ep_read.set_read_timeout(DEFAULT_TIMEOUT);
    let mut ep_write = ep_out.writer(64);
    ep_write.set_write_timeout(DEFAULT_TIMEOUT);
    Ok(DeviceEndpoint {
        in_max_size,
        ep_in: ep_read,
        ep_out: ep_write,
    })
}
async fn handshake(ep: &mut DeviceEndpoint, handshake_magic: &[u8]) -> Result<(), KakikaeError> {
    println!("Trying to handshake (USB)...");
    stdout().flush()?;
    let timer = Instant::now();
    let mut i = 0;
    while i < handshake_magic.len() {
        let current_time = Instant::now();
        if (current_time - timer).as_secs() > 5 {
            return Err(KakikaeError::HandshakeTimeout)
        }
        if let Ok(_) = ep.ep_out.write(&[handshake_magic[i]]) {
            ep.ep_out.flush()?;
            let mut recv = vec![0; ep.in_max_size];
            ep.ep_in.read(&mut recv)?;
            if recv[0] ^ handshake_magic[i] == 0xff {
                i += 1
            }
        } else {
            i = 0;
        }
    }
    println!("Handshake completed (USB)...");
    Ok(())
}

async fn cmd_write_32(
    ep: &mut DeviceEndpoint,
    addr: u32,
    data: &[u32],
) -> Result<(), KakikaeError> {
    echo(ep, &[0xd4], None).await?;
    echo(ep, &addr.to_be_bytes(), Some(4)).await?;
    echo(ep, &(data.len() as u32).to_be_bytes(), Some(4)).await?;
    let status = status_check(ep).await?;
    assert!(status <= 3);
    for val in data {
        echo(ep, &val.to_be_bytes(), None).await?;
    }
    let status = status_check(ep).await?;
    assert!(status <= 0xff);
    Ok(())
}
async fn cmd_read_32(
    ep: &mut DeviceEndpoint,
    addr: u32,
    size: u32,
) -> Result<Vec<u32>, KakikaeError> {
    echo(ep, &[0xd1], None).await?;
    echo(ep, &addr.to_be_bytes(), Some(4)).await?;
    echo(ep, &size.to_be_bytes(), Some(4)).await?;
    let status = status_check(ep).await?;
    assert!(status <= 0xff);

    let mut recv_ints = vec![0u32; size as usize];
    let mut recv = [0u8; 4];
    for _ in 0..size {
        ep.ep_in.read(&mut recv)?;
        recv_ints.push(u32::from_be_bytes(recv))
    }


    let status2 = status_check(ep).await?;
    assert!(status2 <= 0xff);
    Ok(recv_ints)
}
async fn cmd_get_hwcode(ep: &mut DeviceEndpoint) -> Result<u16, KakikaeError> {
    echo(ep, &[0xfd], None).await?;
    let mut recv = [0u8; 2];
    ep.ep_in.read(&mut recv)?;
    let status = status_check(ep).await?;
    assert!(status <= 0xff);

    Ok(u16::from_be_bytes(recv))
}
async fn cmd_send_da(
    ep: &mut DeviceEndpoint,
    addr: u32,
    data: &[u8],
) -> Result<(), KakikaeError> {
    echo(ep, &[0xd7], None).await?;
    echo(ep, &addr.to_be_bytes(), Some(4)).await?;
    echo(ep, &(data.len() as u32).to_be_bytes(), Some(4)).await?;
    echo(ep, &0x100u32.to_be_bytes(), Some(4)).await?; // todo: sig_size
    let status = status_check(ep).await?;
    assert!(status <= 0xff);

    for chunk in data.chunks(64) {
        ep.ep_out.write(chunk)?;
        ep.ep_out.flush()?;
    }
    ep.ep_out.write(&[])?;

    sleep(DEFAULT_TIMEOUT).await;
    let _checksum = status_check(ep).await?; // todo: checksum
    let status = status_check(ep).await?;
    assert!(status <= 0xff);
    Ok(())
}

async fn cmd_jump_da(ep: &mut DeviceEndpoint, addr: u32) -> Result<(), KakikaeError> {
    echo(ep, &[0xd5], None).await?;
    echo(ep, &addr.to_be_bytes(), Some(4)).await?;
    let status = status_check(ep).await?;
    assert_eq!(status, 0);

    Ok(())
}
async fn cmd_boot_pl(ep: &mut DeviceEndpoint) -> Result<(), KakikaeError> {
    echo(ep, &[0xd6], None).await?;
    let status = status_check(ep).await?;
    assert_eq!(status, 0);
    let status = status_check(ep).await?;
    assert_eq!(status, 0);
    Ok(())
}
async fn cmd_register_access(
    ep: &mut DeviceEndpoint,
    direction: u32,
    offset: u32,
    length: u32,
    mut data: Vec<u8>,
    check_status: bool
) -> Result<Vec<u8>, KakikaeError> {
    if let Ok(_) = echo(ep, &[0xda], None).await {
        data.resize(length as usize, 0);
        echo(ep, &direction.to_be_bytes(), Some(4)).await?;
        echo(ep, &offset.to_be_bytes(), Some(4)).await?;
        echo(ep, &length.to_be_bytes(), Some(4)).await?;
        status_check(ep).await?;

        if (direction & 1) != 0 {
            ep.ep_out.write(&data)?;
            ep.ep_out.flush()?;
        } else {
            data = vec![0; length as usize];
            ep.ep_in.read(&mut data)?;
        };

        if check_status {
            status_check(ep).await?;
        }
    } else {
        println!("Failed to send cmd_da")
    }
    Ok(data)
}
async fn echo(ep: &mut DeviceEndpoint, data: &[u8], size: Option<usize>) -> Result<(), KakikaeError> {
    let size = if let Some(size) = size {
        size
    } else {
        data.len()
    };
    let mut data = data.to_vec();
    data.resize(size, 0);
    ep.ep_out.write(&data)?;
    ep.ep_out.flush()?;
    let mut recv = vec![0; size];
    ep.ep_in.read(&mut recv)?;
    if data != recv {
        return Err(KakikaeError::EchoMismatch(data.into(), recv))
    }
    Ok(())
}
async fn status_check(ep: &mut DeviceEndpoint) -> Result<u16, KakikaeError> {
    let mut recv = [0u8; 2];
    ep.ep_in.read(&mut recv)?;
    let result = u16::from_be_bytes(recv);
    if result > 0xFF {
        Err(KakikaeError::StatusError(result))
    } else { 
        Ok(result)
    }
}
#[allow(unused_must_use)]
async fn stage_cmd(ep: &mut DeviceEndpoint, data: u32, await_result: bool) -> Result<(), KakikaeError> {
    let data = data.to_be_bytes();
    ep.ep_out.write(&data)?;
    ep.ep_out.flush()?;
    sleep(SHORT_TIMEOUT).await;
    let mut recv = [0; 4];
    if await_result {
        ep.ep_in.read(&mut recv)?;
        stdout().flush()?;
        if data != recv {
            return Err(KakikaeError::EchoMismatch(data.into(), recv.into()))
        }
    }
    Ok(())
}
async fn stage_write_data(
    ep: &mut DeviceEndpoint,
    data: &[u8],
    location: u32
) -> Result<(), KakikaeError> {
    stage_cmd(ep, location, true).await?;
    stage_cmd(ep, data.len() as u32, true).await?;
    let (chunks, remainder) = data.as_chunks::<64>();
    for chunk in chunks {
        ep.ep_out.write(chunk)?;
        ep.ep_out.flush()?;
        sleep(SHORT_TIMEOUT).await;
    }
    // write remainder as a full chunk
    if !remainder.is_empty() {
        let mut remainder_vec = remainder.to_vec();
        remainder_vec.resize(64, 0);
        ep.ep_out.write(&remainder_vec)?;
        ep.ep_out.flush()?;
    }
    Ok(())
}
