use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

fn last_event() -> mock::Event {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("Event expected")
		.event
}

pub fn before_exchange() {
	assert_ok!(WrapCurrency::mint(Origin::signed(1), W3G, 500 * CENTS));

	assert_ok!(Token::create_instance(Origin::signed(1), [0].to_vec()));
	assert_ok!(Token::create_token(
		Origin::signed(1),
		1,
		1,
		false,
		[0].to_vec()
	));
	assert_ok!(Token::mint(Origin::signed(1), 1, 1, 1, 50000 * CENTS));
}

#[test]
fn create_exchange_works() {
	new_test_ext().execute_with(|| {
		before_exchange();
		run_to_block(10);
		assert_ok!(Dex::create_exchange(Origin::signed(1), W3G, 1));

		assert_eq!(
			last_event(),
			mock::Event::pallet_dex(crate::Event::ExchangeCreated(0, 1)),
		);
	})
}

#[test]
fn add_liquidity_works() {
	new_test_ext().execute_with(|| {
		before_exchange();
		run_to_block(10);

		assert_ok!(Dex::create_exchange(Origin::signed(1), W3G, 1));

		assert_ok!(Dex::add_liquidity(
			Origin::signed(1),
			0,
			1,
			[1].to_vec(),
			[1000 * CENTS].to_vec(),
			[100 * CENTS].to_vec(),
		));
	});
}
