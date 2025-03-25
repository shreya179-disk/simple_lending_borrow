use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;
pub const FEED_ID_SOL_USD: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
pub const FEED_ID_USDC_USD: &str = "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";
pub const MAXIMUM_AGE:u64 = 100; //maximum_age refers to the maximum allowable time difference (in seconds) between the current blockchain time and the timestamp of the price update. If the price data is older than this limit, the function get_price_no_older_than will return an error.