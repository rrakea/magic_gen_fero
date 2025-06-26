const MAX_BLOCKER_ROOK: u64 = u64::pow(2, 14);
const ROOK_OFFSETS: [i8; 4] = [1, -1, 8, -8];

static mut ROOK_MOVEMASK: [[u64; MAX_BLOCKER_ROOK as usize]; 64] =
    [[0; MAX_BLOCKER_ROOK as usize]; 64];

fn main() {
    init_piecemask();
    init_magics();
}

pub fn init_magics() {
    // Rook movemasks
    for sq in 0..64 {
        let premask = unsafe { ROOK_MASKS[sq] };

        for blocker_index in 0..MAX_BLOCKER_ROOK {
            // Calculate all the possible relevant blocker positions
            let blocker = gen_blockers(premask, blocker_index);

            // Calculate all the possible moves
            let mut pos_mv = Vec::new();
            for offset in ROOK_OFFSETS {
                for (i, sq) in (1..8).enumerate() {
                    let new_pos: i8 = sq as i8 + offset * i as i8;
                    if new_pos >= 0
                        || new_pos < 64
                        || no_wrap((sq as i8 + offset * (i as i8 - 1)) as u8, new_pos as u8)
                    {
                        pos_mv.push(new_pos as u8);
                    } else {
                        break;
                    }
                }
            }
            let movemask = one_at(pos_mv);
            unsafe {
                ROOK_MOVEMASK[sq][blocker_index as usize] = movemask;
            }
        }
    }
}

fn gen_blockers(mask: u64, counter: u64) -> u64 {
    let mut res = 0;
    let mut counter_position = 0;

    for b in 0..64 {
        // If the mask has a 1 at position i
        if (mask >> b) & 1 == 1 {
            // We only need to flip the bit in res if
            // the counter actually has a 1 in that position
            if (counter >> counter_position) & 1 == 1 {
                res |= 1 << b;
            }
            // We have consumed a bit from the counter
            counter_position += 1;
        }
    }
    res
}

// This accomodates for knight moves
pub fn no_wrap(a: u8, b: u8) -> bool {
    (a as i16 % 8 - b as i16 % 8).abs() <= 2
}

pub static mut BISHOP_MASKS: [u64; 64] = [0; 64];
pub static mut ROOK_MASKS: [u64; 64] = [0; 64];
pub static mut KNIGHT_MASKS: [u64; 64] = [0; 64];
pub static mut KING_MASKS: [u64; 64] = [0; 64];

pub fn init_piecemask() {
    let bishop_offsets = vec![7, 9, -7, -9];
    let rook_offsets = vec![1, -1, 8, -8];
    let king_offsets = vec![1, -1, 8, -8, 7, -7, 9, -9];
    let knight_offsets = vec![-10, 6, 15, 17, 10, -6, -15, -17];

    unsafe {
        KING_MASKS = mask_from_offset(&king_offsets, 1);
        BISHOP_MASKS = mask_from_offset(&bishop_offsets, 8);
        ROOK_MASKS = mask_from_offset(&rook_offsets, 8);
        KNIGHT_MASKS = mask_from_offset(&knight_offsets, 1);
    }
}

fn mask_from_offset(offset: &Vec<i8>, iterator: i8) -> [u64; 64] {
    let mut mask = [0; 64];

    for sq in 0..64 {
        let mut pos_sq = Vec::new();
        for o in offset {
            for i in 1..=iterator {
                let new_pos = (sq as i8) + (o * i);
                if new_pos >= 0
                    && new_pos < 64
                    && no_wrap(((sq as i8) + (o * (i - 1))) as u8, new_pos as u8)
                {
                    pos_sq.push(new_pos as u8);
                } else {
                    break;
                }
            }
        }
        mask[sq as usize] = one_at(pos_sq);
    }
    mask
}

pub fn one_at(b: Vec<u8>) -> u64 {
    let mut mask = 0;
    for bit in b {
        mask |= 1 << bit;
    }
    mask
}
