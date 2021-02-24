use crate::{Error, mock::*, pallet::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn create_token_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];
        assert_ok!(TokenModule::create_token(&1, true, &uri));

        println!("token: {:?}", Tokens::<Test>::get(&0).unwrap());

        assert_eq!(TokenCount::<Test>::get().unwrap(), 1);
    })
}
