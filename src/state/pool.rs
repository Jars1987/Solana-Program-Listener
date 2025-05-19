use solana_sdk::pubkey::Pubkey;

pub struct Poll {
    pub poll_id: u64,
    pub poll_owner: Pubkey,
    pub poll_name: String,
    pub poll_description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
    pub candidate_winner: Pubkey,
}

impl Poll {
    pub fn try_from_anchor_bytes(data: &[u8]) -> Option<Self> {
        let mut offset = 0;

        if data.len() < offset + 8 {
            return None;
        }
        let poll_id = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        if data.len() < offset + 32 {
            return None;
        }
        let poll_owner = Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());
        offset += 32;

        let (poll_name, len) = read_anchor_string_manual(&data[offset..], 64)?;
        offset += len;

        let (poll_description, len) = read_anchor_string_manual(&data[offset..], 280)?;
        offset += len;

        if data.len() < offset + 8 {
            return None;
        }
        let poll_start = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        if data.len() < offset + 8 {
            return None;
        }
        let poll_end = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        if data.len() < offset + 8 {
            return None;
        }
        let candidate_amount = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        if data.len() < offset + 32 {
            return None;
        }
        let candidate_winner =
            Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());

        Some(Self {
            poll_id,
            poll_owner,
            poll_name,
            poll_description,
            poll_start,
            poll_end,
            candidate_amount,
            candidate_winner,
        })
    }
}

fn read_anchor_string_manual(data: &[u8], max_len: usize) -> Option<(String, usize)> {
    if data.len() < 4 {
        return None;
    }

    let len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    if len > max_len {
        return None;
    }

    if data.len() < 4 + len {
        return None;
    }

    let string_bytes = &data[4..4 + len];
    let s = std::str::from_utf8(string_bytes).ok()?.to_string();

    Some((s, 4 + len))
}
