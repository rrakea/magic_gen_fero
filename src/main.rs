use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

/* Intuition for magic calculation
    We decide a shift length (e.g. 22)
    We generate a array corresponding to that size
    (e.g. 2 ^ 22)
    We generate all the possible moves on an empty board
    We generate all the possible blockers for that square
    We and all the possible moves and the blockers
    (also & the border since it does not matter if there is a piece there)
    We generate all the possible moves
    loop:
        We generate a random magic u64
        We multiply it by the blocker mask
        We shift it by 64 - shift number
        We check that index in the array
            If 0:
                Set it to the possible moves
            If != 0:
                if the result is the same as we have
                    do nothing
                else
                    magic failed -> continue;
        If this works for every blocker:
            magic found!
        Reduce the shift size and search again
*/
const MAGIC_TRIES: u64 = 100000;
const START_SHIFT: u32 = 14;
const EDGE_MASK: u64 = 0;
const MAX_BLOCKER_ROOK: usize = usize::pow(2, 14);
const ROOK_OFFSETS: [i8; 4] = [1, -1, 8, -8];

// This saves the masks of moves on an empty square
static mut ROOK_PREMASK: [u64; 64] = [0; 64];
static mut BISHOP_PREMASK: [u64; 64] = [0; 64];

// The tupel saves the magic (u64) and the shift (u32)
static mut ROOK_MAGIC: [(u64, u32); 64] = [(0, 0); 64];
static mut BISHOP_MAGIC: [(u64, u32); 64] = [(0, 0); 64];

// This saves all the possible blockers from every square
static mut ROOK_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];
static mut BISHOP_BLOCKERS: [u64; 64] = [0; 64];

static mut ROOK_MOVE_WITH_BLOCKERS: [[u64; MAX_BLOCKER_ROOK]; 64] = [[0; MAX_BLOCKER_ROOK]; 64];

fn main() {
    // Sets ROOK_BLOCKERS and ROOK_MOVE_WITH_BLOCKERS
    init_piecemask();
    // Sets ROOK_PREMASK
    init_movemasks();

    let test_premask = unsafe { ROOK_PREMASK[32] };
    let test_blocker = unsafe { ROOK_BLOCKERS[32][100] };
    let test_movemask = unsafe { ROOK_MOVE_WITH_BLOCKERS[32][100] };
    println!(
        "Testcase 1: \npremask: {:064b}\nblocker: {:064b}\n moves: {:064b}",
        test_premask, test_blocker, test_movemask
    );
    let test_premask = unsafe { ROOK_PREMASK[28] };
    let test_blocker = unsafe { ROOK_BLOCKERS[28][15] };
    let test_movemask = unsafe { ROOK_MOVE_WITH_BLOCKERS[28][15] };
    println!(
        "Testcase 2: \npremask: {:064b}\nblocker: {:064b}\n moves: {:064b}",
        test_premask, test_blocker, test_movemask
    );

    let mut rng = Pcg64::from_os_rng();

    for sq in 0..64 {
        let move_mask = unsafe { ROOK_PREMASK[sq] };
        let shift_len = START_SHIFT;
        'shifts: for shift in (12..shift_len).rev() {
            'magics: for _ in 0..MAGIC_TRIES {
                // 2 ^ shift is the max amount we can reach
                // since we truncate our index to that value
                let mut move_lookup = vec![0u64; usize::pow(2, shift)];

                // Using a lot of random numbers and anding them together tends to create a lower number
                // This tends to create better magics
                let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

                'blockers: for (blocker_index, blocker) in
                    unsafe { ROOK_BLOCKERS[sq] }.iter().enumerate()
                {
                    let blocker_mask = move_mask & blocker;
                    let index = (blocker_mask * magic) >> (64 - shift);
                    let move_with_blocker = unsafe { ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index] };
                    if move_lookup[index as usize] == 0 {
                        move_lookup[index as usize] = move_with_blocker;
                    } else {
                        // There is something already at that index
                        if move_lookup[index as usize] == move_with_blocker {
                            continue 'blockers;
                        } else {
                            continue 'magics;
                        }
                    }
                }

                // We have gone through all of the blockers and all works
                // -> The magic works!
                unsafe {
                    ROOK_MAGIC[sq] = (magic, shift);
                };
                continue 'shifts;
            }
            // If we havent update the magic this shift value
            // We assume there is no magic possible at this/ a smaller shift value
            if unsafe { ROOK_MAGIC[sq].1 } != shift {
                break 'shifts;
            }
        }
    }
    for (sq, magic) in unsafe { ROOK_MAGIC }.iter().enumerate() {
        println!("Sq: {}, Magic: {:064b}, Shift {}", sq, magic.0, magic.1);
    }
}

fn init_movemasks() {
    // Rook movemasks
    for sq in 0..64 {
        let premask = unsafe { ROOK_PREMASK[sq] };

        for blocker_index in 0..MAX_BLOCKER_ROOK {
            // Calculate all the possible relevant blocker positions
            let blocker = gen_blockers(premask, blocker_index as u64);
            unsafe {
                ROOK_BLOCKERS[sq][blocker_index] = blocker;
            };

            // Calculate all the possible moves
            let mut pos_mv = Vec::new();
            'offset: for offset in ROOK_OFFSETS {
                let mut found_blocker = false;
                for i in 1..8 {
                    let new_pos: i8 = sq as i8 + offset * i as i8;
                    if new_pos >= 0
                        && new_pos < 64
                        && no_wrap((sq as i8 + offset * (i as i8 - 1)) as u8, new_pos as u8)
                        && !found_blocker
                    {
                        // There is a blocker there:
                        if (blocker >> new_pos) & 1 == 1 {
                            found_blocker = true;
                        }
                        pos_mv.push(new_pos as u8);
                    } else {
                        continue 'offset;
                    }
                }
            }
            let movemask = one_at(pos_mv);
            unsafe {
                ROOK_MOVE_WITH_BLOCKERS[sq][blocker_index] = movemask;
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

pub fn init_piecemask() {
    let bishop_offsets = vec![7, 9, -7, -9];
    let rook_offsets = vec![1, -1, 8, -8];

    unsafe {
        BISHOP_PREMASK = mask_from_offset(&bishop_offsets, 8);
        ROOK_PREMASK = mask_from_offset(&rook_offsets, 8);
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
