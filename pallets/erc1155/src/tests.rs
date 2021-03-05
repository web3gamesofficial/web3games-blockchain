use crate::{mock::*, pallet::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_token_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];
        let data = vec![0, 1];
        assert_ok!(Erc1155::do_create_tao(&1, data));

        assert_noop!(
            Erc1155::do_create_token(&1, 1, 2, true, uri),
            Error::<Test>::InvalidTaoId
        );
        println!("token: {:?}", Tokens::<Test>::get(1, 2));
    })
}

#[test]
fn test_create_token_not_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];

        assert_noop!(
            Erc1155::do_create_token(&1, 1, 2, true, uri),
            Error::<Test>::InvalidTaoId
        );
        println!("token: {:?}", Tokens::<Test>::get(1, 2));
    })
}

#[test]
fn test_create_tao_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Erc1155::do_create_tao(&1, data));
    })
}

#[test]
#[ignore]
fn test_do_set_approval_for_all() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_mint() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_batch_mint() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_burn() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_batch_burn() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_transfer_from() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_do_batch_transfer_from() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_approved_or_owner() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_is_approved_for_all() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_balance_of() {
    new_test_ext().execute_with(|| {})
}

#[test]
#[ignore]
fn test_balance_of_batch() {
    new_test_ext().execute_with(|| {})
}
