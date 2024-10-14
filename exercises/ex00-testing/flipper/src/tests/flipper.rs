use super::mock::*;
use frame_support::{assert_noop, assert_ok, assert_err};

type Error = crate::Error::<TestRuntime>;

#[test]
fn set_value_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(RuntimeOrigin::signed(ALICE), false));
		assert_eq!(Flipper::value(), Some(false));
	});
}

#[test]
fn set_value_err_already_set() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(RuntimeOrigin::signed(ALICE), true));
		assert_noop!(Flipper::set_value(RuntimeOrigin::signed(ALICE), true), Error::AlreadySet);
	});
}

#[test]
fn flip_value_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(Flipper::set_value(RuntimeOrigin::signed(ALICE), false));
		assert_ok!(Flipper::flip_value(RuntimeOrigin::signed(ALICE)));
		assert_eq!(Flipper::value(), Some(true));
	});
}

#[test]
fn flip_value_ko() {
	new_test_ext().execute_with(|| {
		assert_err!(Flipper::flip_value(RuntimeOrigin::signed(ALICE)), Error::NoneValue);
	});
}
