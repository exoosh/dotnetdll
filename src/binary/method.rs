use super::il;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};
use scroll_buffer::DynamicBuffer;

#[derive(Debug)]
pub enum Header {
    Tiny {
        size: usize,
    },
    Fat {
        more_sects: bool,
        init_locals: bool,
        max_stack: u16,
        size: usize,
        local_var_sig_tok: u32,
    },
}

impl TryFromCtx<'_> for Header {
    type Error = scroll::Error;

    fn try_from_ctx(from: &[u8], (): ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let b1: u8 = from.gread_with(offset, scroll::LE)?;
        Ok((
            if b1 & 0b11 == 2 {
                Header::Tiny {
                    size: (b1 >> 2) as usize,
                }
            } else {
                // second byte isn't relevant right now
                // see ECMA-335, II.25.4.3 (page 285)
                *offset += 1;

                Header::Fat {
                    more_sects: check_bitmask!(b1, 0x8),
                    init_locals: check_bitmask!(b1, 0x10),
                    max_stack: from.gread_with(offset, scroll::LE)?,
                    size: from.gread_with::<u32>(offset, scroll::LE)? as usize,
                    local_var_sig_tok: from.gread_with(offset, scroll::LE)?,
                }
            },
            *offset,
        ))
    }
}
impl TryIntoCtx<(), DynamicBuffer> for Header {
    type Error = scroll::Error;

    fn try_into_ctx(self, into: &mut DynamicBuffer, (): ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;

        use Header::*;
        match self {
            Tiny { size } => {
                into.gwrite_with(((size as u8) << 2) | 0x2, offset, scroll::LE)?;
            }
            Fat {
                more_sects,
                init_locals,
                max_stack,
                size,
                local_var_sig_tok,
            } => {
                let mut flags: u16 = 0x3;
                if more_sects {
                    flags |= 0x8;
                }
                if init_locals {
                    flags |= 0x10;
                }

                let header_size: u16 = 3;

                bitfield::bitfield! {
                    pub struct FlagsSize(u16);
                    flags, set_flags: 11, 0;
                    size, set_size: 15, 12;
                }

                let mut fields = FlagsSize(0);
                fields.set_flags(flags);
                fields.set_size(header_size);

                into.gwrite_with(fields.0, offset, scroll::LE)?;

                into.gwrite_with(max_stack, offset, scroll::LE)?;
                into.gwrite_with(size as u32, offset, scroll::LE)?;
                into.gwrite_with(local_var_sig_tok, offset, scroll::LE)?;
            }
        }

        Ok(*offset)
    }
}

#[derive(Debug, Pread)]
pub struct Exception {
    pub flags: u32,
    pub try_offset: u32,
    pub try_length: u32,
    pub handler_offset: u32,
    pub handler_length: u32,
    pub class_token_or_filter: u32,
}
impl TryIntoCtx<(), DynamicBuffer> for Exception {
    type Error = scroll::Error;

    fn try_into_ctx(self, into: &mut DynamicBuffer, (): ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;

        if let (Ok(tlen), Ok(hlen), Ok(toff), Ok(hoff)) = (
            u8::try_from(self.try_length),
            u8::try_from(self.handler_length),
            u16::try_from(self.try_offset),
            u16::try_from(self.handler_offset),
        ) {
            into.gwrite_with(self.flags as u8, offset, scroll::LE)?;
            into.gwrite_with(toff, offset, scroll::LE)?;
            into.gwrite_with(tlen, offset, scroll::LE)?;
            into.gwrite_with(hoff, offset, scroll::LE)?;
            into.gwrite_with(hlen, offset, scroll::LE)?;
        } else {
            into.gwrite_with(self.flags, offset, scroll::LE)?;
            into.gwrite_with(self.try_offset, offset, scroll::LE)?;
            into.gwrite_with(self.try_length, offset, scroll::LE)?;
            into.gwrite_with(self.handler_offset, offset, scroll::LE)?;
            into.gwrite_with(self.handler_length, offset, scroll::LE)?;
        }

        into.gwrite_with(self.class_token_or_filter, offset, scroll::LE)?;

        Ok(*offset)
    }
}

#[derive(Debug)]
pub enum SectionKind {
    Exceptions(Vec<Exception>),
    Unrecognized { is_fat: bool, length: usize },
}

#[derive(Debug)]
pub struct DataSection {
    pub section: SectionKind,
    pub more_sections: bool,
}

impl TryFromCtx<'_> for DataSection {
    type Error = scroll::Error;

    fn try_from_ctx(from: &[u8], (): ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let kind: u8 = from.gread_with(offset, scroll::LE)?;
        let is_exception = check_bitmask!(kind, 1);
        let is_fat = check_bitmask!(kind, 0x40);
        let more_sections = check_bitmask!(kind, 0x80);

        let length = if is_fat {
            let mut bytes = [0_u8; 3];
            from.gread_inout_with(offset, &mut bytes, scroll::LE)?;
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
        } else {
            from.gread_with::<u8>(offset, scroll::LE)? as u32
        } as usize;

        let section = if is_exception {
            if !is_fat {
                // 2 bytes of padding on small exception headers
                *offset += 2;
            }

            let n = (length - 4) / if is_fat { 24 } else { 12 };

            let exceptions = (0..n)
                .map(|_| {
                    if is_fat {
                        from.gread_with(offset, scroll::LE)
                    } else {
                        Ok(Exception {
                            flags: from.gread_with::<u16>(offset, scroll::LE)? as u32,
                            try_offset: from.gread_with::<u16>(offset, scroll::LE)? as u32,
                            try_length: from.gread_with::<u8>(offset, scroll::LE)? as u32,
                            handler_offset: from.gread_with::<u16>(offset, scroll::LE)? as u32,
                            handler_length: from.gread_with::<u8>(offset, scroll::LE)? as u32,
                            class_token_or_filter: from.gread_with(offset, scroll::LE)?,
                        })
                    }
                })
                .collect::<Result<_, _>>()?;

            SectionKind::Exceptions(exceptions)
        } else {
            *offset += length;
            SectionKind::Unrecognized { is_fat, length }
        };

        Ok((DataSection { section, more_sections }, *offset))
    }
}
impl TryIntoCtx<(), DynamicBuffer> for DataSection {
    type Error = scroll::Error;

    fn try_into_ctx(self, into: &mut DynamicBuffer, (): ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;

        use SectionKind::*;

        let mut kind: u8 = if self.more_sections { 0x80 } else { 0x0 };

        let (is_fat, length) = match &self.section {
            Exceptions(es) => {
                kind |= 0x1;

                let should_be_fat = !es.iter().all(|e| {
                    e.try_length < 256 && e.handler_length < 256 && e.try_offset < 65536 && e.handler_offset < 65536
                });

                (should_be_fat, (if should_be_fat { 24 } else { 12 } * es.len()) + 4)
            }
            Unrecognized { is_fat, length } => (*is_fat, *length),
        };

        if is_fat {
            kind |= 0x40;
        }

        into.gwrite_with(kind, offset, scroll::LE)?;

        if is_fat {
            let bytes = (length as u32).to_le_bytes();
            for b in &bytes[..3] {
                into.gwrite_with(b, offset, scroll::LE)?;
            }
        } else {
            into.gwrite_with(length as u8, offset, scroll::LE)?;
        }

        match self.section {
            Exceptions(e) => {
                // small exception table requires 2 bytes padding
                if !is_fat {
                    into.gwrite_with(0_u16, offset, scroll::LE)?;
                }

                for clause in e {
                    if is_fat {
                        into.gwrite(clause, offset)?;
                    } else {
                        into.gwrite_with(clause.flags as u16, offset, scroll::LE)?;
                        into.gwrite_with(clause.try_offset as u16, offset, scroll::LE)?;
                        into.gwrite_with(clause.try_length as u8, offset, scroll::LE)?;
                        into.gwrite_with(clause.handler_offset as u16, offset, scroll::LE)?;
                        into.gwrite_with(clause.handler_length as u8, offset, scroll::LE)?;
                        into.gwrite_with(clause.class_token_or_filter, offset, scroll::LE)?;
                    }
                }
            }
            Unrecognized {
                length: section_length, ..
            } => {
                // just skip any unknown sections
                *offset += section_length;
            }
        }

        Ok(*offset)
    }
}

#[derive(Debug)]
pub struct Method {
    pub header: Header,
    pub body: Vec<il::Instruction>,
    pub data_sections: Vec<DataSection>,
}

impl TryFromCtx<'_> for Method {
    type Error = scroll::Error;

    fn try_from_ctx(from: &[u8], (): ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let header = from.gread(offset)?;

        let body_size = match header {
            Header::Tiny { size } | Header::Fat { size, .. } => size,
        };

        let body_bytes: &[u8] = from.gread_with(offset, body_size)?;
        let mut body = vec![];
        let mut body_offset = 0;
        while body_offset < body_size {
            body.push(body_bytes.gread(&mut body_offset)?);
        }

        let mut data_sections = vec![];

        if let Header::Fat {
            more_sects: mut has_next,
            ..
        } = header
        {
            // align to next 4-byte boundary
            *offset = crate::utils::round_up_to_4(*offset).0;

            while has_next {
                let sec: DataSection = from.gread(offset)?;
                has_next = sec.more_sections;
                data_sections.push(sec);
            }
        }

        Ok((
            Method {
                header,
                body,
                data_sections,
            },
            *offset,
        ))
    }
}
impl TryIntoCtx<(), DynamicBuffer> for Method {
    type Error = scroll::Error;

    fn try_into_ctx(self, into: &mut DynamicBuffer, (): ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;

        into.gwrite(self.header, offset)?;

        for i in self.body {
            into.gwrite(i, offset)?;
        }

        // align to next 4-byte boundary
        *offset = crate::utils::round_up_to_4(*offset).0;

        for d in self.data_sections {
            into.gwrite(d, offset)?;
        }

        Ok(*offset)
    }
}
