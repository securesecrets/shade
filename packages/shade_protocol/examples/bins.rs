use ethnum::U256;
use shade_protocol::liquidity_book::lb_libraries::price_helper::PriceHelper;

fn main() {
    let price = U256::from_words(1u128, 0u128);
    let bin_step = 100u16;

    let id = PriceHelper::get_id_from_price(price, bin_step).unwrap();
    println!("{}", id);
}
