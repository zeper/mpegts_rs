pub enum TransportScramblingControl {
    NOTSCRAMBLED,
    SCRAMBLED,
    UNKNOWN,
}

#[derive(Debug)]
pub enum AdaptationFieldControl	{
    NoAdaptationfieldPayloadOnly,
    AdaptationfieldOnlyNoPayload,
    AdaptationfieldFollowedByPayload,
    Unknown,
}

// FIXED DATA SIZE
const PCR_LENGTH:usize = 6;
const OPCR_LENGTH:usize = 6;
const SLICE_COUNT_LENGTH:usize = 1;

// AdaptationFiled in Packet
pub struct AdaptationFiled<'b> {
    buf: &'b[u8],
}

impl <'b> AdaptationFiled<'b> {

    pub fn length(&self) -> usize {
        self.buf[0].into()
    }

    pub fn discontinuity_indicator(&self) -> bool {
        self.buf[1] & 0x80 != 0
    }

    pub fn random_access_indicator(&self) -> bool {
        self.buf[1] & 0x40 != 0
    }

    pub fn elementary_stream_priority_indicator(&self) -> bool {
        self.buf[1] & 0x20 != 0
    }

    pub fn pcr_flag(&self) -> bool {
        self.buf[1] & 0x10 != 0
    }

    pub fn opcr_flag(&self) -> bool {
        self.buf[1] & 0x08 != 0
    }

    pub fn splicing_point_flag(&self) -> bool {
        self.buf[1] & 0x04 != 0
    }

    pub fn transport_private_data_flag(&self) -> bool {
        self.buf[1] & 0x02 != 0
    }

    pub fn adaptation_field_extension_flag(&self) -> bool {
        self.buf[1] & 0x01 != 0
    }

    pub fn pcr_bytes(&self) -> Option<&'b[u8]> {
        let offset = 2;
        if self.pcr_flag() {
            Some(&self.buf[offset..offset+PCR_LENGTH])
        }
        else {
            None
        }
    }

    pub fn opcr_bytes(&self) -> Option<&'b[u8]> {
        let mut offset = 2;
        if self.pcr_flag() {
            offset += PCR_LENGTH;
        }

        if self.opcr_flag() {
            Some(&self.buf[offset..offset+OPCR_LENGTH])
        }
        else {
            None
        }
    }

    pub fn splice_countdown_byte(&self) -> Option<&'b[u8]> {
        let mut offset = 2;
        if self.pcr_flag() {
            offset += PCR_LENGTH;
        }
        if self.opcr_flag() {
            offset += OPCR_LENGTH;
        }

        if self.splicing_point_flag() {
            Some(&self.buf[offset..offset+1])
        }
        else {
            None
        }
    }

    pub fn transport_private_date_bytes(&self) -> Option<&'b[u8]> {
        let mut offset = 2;
        if self.pcr_flag() {
            offset += PCR_LENGTH;
        }
        if self.opcr_flag() {
            offset += OPCR_LENGTH;
        }
        if self.splicing_point_flag() {
            offset += SLICE_COUNT_LENGTH;
        }

        if self.transport_private_data_flag() {
            let data_length:usize = self.buf[offset].into();
            Some(&self.buf[offset..offset+1+data_length])
        }
        else {
            None
        }
    }

    pub fn adaptation_extension_bytes(&self) -> Option<&'b[u8]> {
        let mut offset = 2;
        if self.pcr_flag() {
            offset += PCR_LENGTH;
        }
        if self.opcr_flag() {
            offset += OPCR_LENGTH;
        }
        if self.splicing_point_flag() {
            offset += SLICE_COUNT_LENGTH;
        }
        if self.transport_private_data_flag() {
            let private_data_len:usize = self.buf[offset].into();
            offset += 1;
            offset += private_data_len;
        }

        if self.adaptation_field_extension_flag() {
            let length = self.buf[offset] as usize;
            Some(&self.buf[offset..offset+1+length])
        }
        else {
            None
        }
    }
}



// TS Packet
pub struct Packet<'b> {
    buf: &'b[u8],
}

impl<'b> Packet<'b> {
    pub const SYNC_BYTE:u8 = 0x47;
    pub const SIZE:usize = 188;

    pub fn is_sync_byte(buf:u8)-> bool {
        buf == Self::SYNC_BYTE
    }

    pub fn new(buf: &'b [u8]) -> Packet<'b> {
        assert_eq!(buf.len(), Self::SIZE);
        assert!(Packet::is_sync_byte(buf[0]));
        Packet { buf }
    }

    pub fn transport_error_indicator(&self) -> bool {
        self.buf[1] & 0x80 != 0
    }

    pub fn payload_unit_start_indicator(&self) -> bool {
        self.buf[1] & 0x40 != 0
    }
    
    pub fn transport_priority(&self) -> bool {
        self.buf[1] & 0x20 != 0
    }

    pub fn pid(&self) -> u16 {
        ((self.buf[1] as u16) & 0x1f) << 8 | ((self.buf[2] as u16) & 0xff)
    }

    pub fn transport_scrambling_control(&self) -> TransportScramblingControl {
        let v = (self.buf[3] & 0xc0) >> 6;
        if v == 0 {
            TransportScramblingControl::NOTSCRAMBLED
        }
        else if v & 0x02 != 0 {
            TransportScramblingControl::SCRAMBLED
        }
        else {
            TransportScramblingControl::UNKNOWN
        }
    }

    pub fn adaptation_filed_control(&self) -> AdaptationFieldControl {
        let v = (self.buf[3] & 0x30) >> 4;
        match v {
            1 => AdaptationFieldControl::NoAdaptationfieldPayloadOnly,
            2 => AdaptationFieldControl::AdaptationfieldOnlyNoPayload,
            3 => AdaptationFieldControl::AdaptationfieldFollowedByPayload,
            _ => AdaptationFieldControl::Unknown,
        }
    }

    pub fn adaptation_filed(&self) -> Option<AdaptationFiled<'b>> {
        let control = self.adaptation_filed_control();

        match control {
            AdaptationFieldControl::NoAdaptationfieldPayloadOnly | 
            AdaptationFieldControl::Unknown =>
            None,
            AdaptationFieldControl::AdaptationfieldOnlyNoPayload |
            AdaptationFieldControl::AdaptationfieldFollowedByPayload => 
            Some(AdaptationFiled { buf: &self.buf[..] }),
        }
    }

    pub fn adaptation_filed_length(&self) -> usize {
        match self.adaptation_filed() {
            Some(a) => a.length(),
            None => 0,
        }
    }

    pub fn payload_psi(&self) -> Option<&'b[u8]> {
        let mut offset = 4;
        offset += self.adaptation_filed_length();
        if self.payload_unit_start_indicator() {
            offset += 1;
        }
        Some(&self.buf[offset..])
    }

    pub fn continuity_counter(&self) -> u8 {
        self.buf[3] & 0x0f
    }

    //pub fn payload(&self) -> &'b [u8]
}


// PSI
struct Pat<'b> {
    buf: &'b[u8],
}

impl <'b> Pat<'b> {
}

pub struct Section<'b> {
    buf: &'b[u8],
}

impl <'b> Section<'b> {

    pub fn new(buf:&'b[u8]) -> Section {
        Section {
            buf
        }
    }

    pub fn table_id(&self) -> u8 {
        self.buf[0]
    }

    pub fn section_syntax_indicator(&self) -> bool {
        self.buf[1] & 0x80 != 0
    }
    pub fn private_indicator(&self) -> bool {
        self.buf[1] & 0x40 != 0
    }

    pub fn section_length(&self) -> u16 {
        ((self.buf[1] & 0x0f) as u16) << 8 | self.buf[2] as u16
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
