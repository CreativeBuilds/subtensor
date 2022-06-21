mod mock;
use mock::*;
use pallet_rpool::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;


/***************************
  pub fn set_weights() tests
*****************************/

// This does not produce the expected result
#[test]
fn test_set_weights_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let dests = vec![1, 1];
		let weights = vec![1, 1];

		let call = Call::Rpool(RpoolCall::set_weights{dests, weights});

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}


/**
* This test the situation where user tries to set weights, but the vecs are empty.
* After setting the weights, the wi
*/
#[test]
fn set_weights_ok_no_weights() {
	new_test_ext().execute_with(|| {

		// == Intial values ==
		let hotkey_account_id:u64 = 55; // Arbitrary number
		let initial_stake = 10000;

		let weights_keys : Vec<u32> = vec![];
		let weight_values : Vec<u32> = vec![];

		// == Expectations ==
		let expect_stake:u64 = 10000; // The stake for the neuron should remain the same
		let expect_total_stake:u64 = 10000; // The total stake should remain the same

		// Let's subscribe a new neuron to the chain
		let neuron = register_ok_neuron( hotkey_account_id, 66);

		// Let's give it some stake.
		Rpool::add_stake_to_neuron_hotkey_account(neuron.uid, initial_stake);

		// Dispatch a signed extrinsic, setting weights.
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);
		assert_ok!(Rpool::set_weights(Origin::signed(hotkey_account_id), weights_keys, weight_values));
		assert_eq!(Rpool::get_weights_for_neuron(&neuron), vec![u32::max_value()]);
		assert_eq!(Rpool::get_stake_of_neuron_hotkey_account_by_uid(neuron.uid), expect_stake);
		assert_eq!(Rpool::get_total_stake(), expect_total_stake);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);

	});
}

#[test]
fn test_priority_increments() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id:u64 = 55; // Arbitrary number
		let neuron = register_ok_neuron( hotkey_account_id, hotkey_account_id );
		Rpool::add_stake_to_neuron_hotkey_account( neuron.uid, 2 );
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);
		assert_ok!(Rpool::set_weights(Origin::signed(hotkey_account_id), vec![], vec![]));
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 1);
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 2);
		assert_ok!(Rpool::set_weights(Origin::signed(hotkey_account_id), vec![], vec![]));
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 1);
		Rpool::add_stake_to_neuron_hotkey_account( neuron.uid, 32 );
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 6);
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 11);
		assert_ok!(Rpool::set_weights(Origin::signed(hotkey_account_id), vec![], vec![]));
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 0);
        step_block (1);
		assert_eq!(Rpool::get_neuron_for_uid( neuron.uid ).priority, 5);
	});
}

#[test]
fn test_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
    	let _neuron = register_ok_neuron( 666, 77);

		let weights_keys: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5]; // Uneven sizes

		let result = Rpool::set_weights(Origin::signed(666), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

#[test]
fn test_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
    	let _neuron = register_ok_neuron( 666, 77);
		let weights_keys: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8];

		let result = Rpool::set_weights(Origin::signed(666), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

#[test]
fn test_no_signature() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<u32> = vec![];
		let weight_values: Vec<u32> = vec![];

		let result = Rpool::set_weights(Origin::none(), weights_keys, weight_values);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

#[test]
fn test_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u32> = vec![1, 2, 3, 4, 5, 6];

		let result = Rpool::set_weights(Origin::signed(1), weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::NotRegistered.into()));
	});
}


#[test]
fn test_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {

        let _neuron = register_ok_neuron( 55, 66);
		let weight_keys : Vec<u32> = vec![99999]; // Does not exist
		let weight_values : Vec<u32> = vec![88]; // random value

		let result = Rpool::set_weights(Origin::signed(55), weight_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));

	});
}

#[test]
fn test_set_weight_not_enough_values() {
	new_test_ext().execute_with(|| {
        let _neuron = register_ok_neuron_with_nonce(1, 2, 100000);
		let _neuron = register_ok_neuron_with_nonce(3, 4, 300000);
		Rpool::set_min_allowed_weights(2);

		// Should fail because we are only setting a single value and its not the self weight.
		let weight_keys : Vec<u32> = vec![1]; // not weight. 
		let weight_values : Vec<u32> = vec![88]; // random value.
		let result = Rpool::set_weights(Origin::signed(1), weight_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::NotSettingEnoughWeights.into()));

		// Shouldnt fail because we setting a single value but it is the self weight.
		let weight_keys : Vec<u32> = vec![0]; // self weight.
		let weight_values : Vec<u32> = vec![88]; // random value.
		assert_ok!( Rpool::set_weights(Origin::signed(1), weight_keys, weight_values)) ;

		// Should pass because we are setting enough values.
		let weight_keys : Vec<u32> = vec![0, 1]; // self weight. 
		let weight_values : Vec<u32> = vec![10, 10]; // random value.
		Rpool::set_min_allowed_weights(1);
		assert_ok!( Rpool::set_weights(Origin::signed(1), weight_keys, weight_values)) ;
	});
}

#[test]
fn test_set_weight_max_min_ratio_to_high() {
	new_test_ext().execute_with(|| {

        let _neuron = register_ok_neuron(1, 11);
		let _neuron = register_ok_neuron(2, 22);

		// max allowed ratio is ok.
		let weight_keys : Vec<u32> = vec![0, 1];
		let weight_values : Vec<u32> = vec![10, 40];		
		assert_ok!( Rpool::set_weights(Origin::signed(1), weight_keys, weight_values)) ;
		
		// Ratio is to high.
		let weight_keys : Vec<u32> = vec![0, 1];
		let weight_values : Vec<u32> = vec![10, 40];		
		Rpool::set_max_allowed_max_min_ratio(2);
		let result = Rpool::set_weights(Origin::signed(1), weight_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::MaxAllowedMaxMinRatioExceeded.into()));

		// Meet ratio.
		let weight_keys : Vec<u32> = vec![0, 1];
		let weight_values : Vec<u32> = vec![10, 20];		
		assert_ok!( Rpool::set_weights(Origin::signed(1), weight_keys, weight_values)) ;

		// Ratio limit is lowered
		let weight_keys : Vec<u32> = vec![0, 1];
		let weight_values : Vec<u32> = vec![10, 40];		
		Rpool::set_max_allowed_max_min_ratio(4);
		assert_ok!( Rpool::set_weights(Origin::signed(1), weight_keys, weight_values) );

	});
}







