use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_exchange_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert!(Dex::exchanges(0).is_none());
        assert_ok!(Dex::create_exchange(Origin::signed(1), 1));

        assert!(Dex::exchanges(0).is_some());

        assert_eq!(
            System::events()
                .into_iter()
                .map(|r| r.event)
                .filter_map(|e| {
                    if let TestEvent::dex(inner) = e {
                        Some(inner)
                    } else {
                        None
                    }
                })
                .last()
                .unwrap(),
            RawEvent::ExchangeCreated(0, 1),
        );
    })
}

pub fn before_exchange() {
    Instance::create_instance(Origin::signed(1), [0].to_vec());
    Instance::create_instance_item(Origin::signed(1), 0, false, [0].to_vec());
    assert!(Token::tokens(0).is_some());

    Currency::create(Origin::signed(1), [0].to_vec());
    Currency::mint(Origin::signed(1), 0, 2000, 1);
    assert!(Currency::currencies(0).is_some());

    Dex::create_exchange(Origin::signed(1), 0);
    assert!(Dex::exchanges(0).is_some());
}

#[test]
fn add_liquidity_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        run_to_block(10);

        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            2,
            [0].to_vec(),
            [100].to_vec(),
            [1000].to_vec(),
            20
        ));
    });
}
