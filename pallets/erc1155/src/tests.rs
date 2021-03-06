use crate::{mock::*, pallet::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_token_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Erc1155::do_create_instance(&1, data));

        println!("token: {:?}", Tokens::<Test>::get(1, 2));
    })
}

#[test]
fn test_create_token_not_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];

        assert_noop!(
            Erc1155::do_create_token(&1, 1, 2, true, uri),
            Error::<Test>::InvalidInstanceId
        );
        println!("token: {:?}", Tokens::<Test>::get(1, 2));
    })
}

#[test]
fn test_create_instance_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Erc1155::do_create_instance(&1, data));
    })
}

#[test]
// #[ignore]
fn test_do_set_approval_for_all() {
    new_test_ext().execute_with(|| {
        assert_ok!(Erc1155::do_set_approval_for_all(&1, &2, true));
        assert_ok!(Erc1155::do_set_approval_for_all(&1, &2, false));
    })
}

#[test]
// #[ignore]
fn test_do_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Erc1155::do_mint(&1, 1, 2, 100));
    })
}

#[test]
// #[ignore]
fn test_do_batch_mint() {
    new_test_ext().execute_with(|| {
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Erc1155::do_batch_mint(&1, 1, token_ids, amounts));
    })
}

#[test]
// #[ignore]
fn test_do_burn_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Erc1155::do_mint(&1, 1, 2, 100));
        assert_ok!(Erc1155::do_burn(&1, 1, 2, 100));
    })
}

#[test]
// #[ignore]
fn test_do_batch_burn() {
    new_test_ext().execute_with(|| {
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Erc1155::do_batch_mint(&1, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_burn(&1, 1, token_ids, amounts));
    })
}

#[test]
// #[ignore]
fn test_do_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Erc1155::do_mint(&1, 1, 2, 100));
        assert_ok!(Erc1155::do_transfer_from(&1,&2, 1, 2, 100));
    })
}

#[test]
// #[ignore]
fn test_do_batch_transfer_from() {
    new_test_ext().execute_with(|| {
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Erc1155::do_batch_mint(&1, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_transfer_from(&1, &2, 1, token_ids, amounts));
    })
}

#[test]
// #[ignore]
fn test_approved_or_owner() {
    new_test_ext().execute_with(|| {
        assert_eq!(Erc1155::approved_or_owner(&1, &2), false);
    })
}

#[test]
// #[ignore]
fn test_is_approved_for_all() {
    new_test_ext().execute_with(|| {
        assert_eq!(Erc1155::is_approved_for_all(&1, &2), false);
    })
}

#[test]
// #[ignore]
fn test_balance_of() {
    new_test_ext().execute_with(|| {
        assert_ok!(Erc1155::do_mint(&1, 1, 2, 100));
        assert_eq!(Erc1155::balance_of(&1, 1, 2), 100);
    })
}

#[test]
// #[ignore]
fn test_balance_of_batch_token_ids_sample() {
    new_test_ext().execute_with(|| {
        let token_ids = vec![1; 3];
        let amounts = vec![100; 3];

        assert_ok!(Erc1155::do_batch_mint(&1, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_mint(&2, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_mint(&3, 1, token_ids.clone(), amounts.clone()));

        let account = vec![1, 2, 3];
        assert_eq!(Erc1155::balance_of_batch(&account, 1, token_ids).unwrap(), vec![300; 3]);
    })
}

#[test]
// #[ignore]
fn test_balance_of_batch_token_ids_not_sample() {
    new_test_ext().execute_with(|| {
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];

        assert_ok!(Erc1155::do_batch_mint(&1, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_mint(&2, 1, token_ids.clone(), amounts.clone()));
        assert_ok!(Erc1155::do_batch_mint(&3, 1, token_ids.clone(), amounts.clone()));

        let account = vec![1, 2, 3];
        assert_eq!(Erc1155::balance_of_batch(&account, 1, token_ids).unwrap(), amounts);
    })
}
