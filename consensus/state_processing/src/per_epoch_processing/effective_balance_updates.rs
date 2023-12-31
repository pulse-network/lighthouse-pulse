use super::errors::EpochProcessingError;
use safe_arith::SafeArith;
use types::beacon_state::BeaconState;
use types::chain_spec::ChainSpec;
use types::{BeaconStateError, EthSpec};

pub fn process_effective_balance_updates<T: EthSpec>(
    state: &mut BeaconState<T>,
    spec: &ChainSpec,
) -> Result<(), EpochProcessingError> {
    let hysteresis_increment = spec
        .effective_balance_increment
        .safe_div(spec.hysteresis_quotient)?;
    let downward_threshold = hysteresis_increment.safe_mul(spec.hysteresis_downward_multiplier)? as u128;
    let upward_threshold = hysteresis_increment.safe_mul(spec.hysteresis_upward_multiplier)? as u128;
    let (validators, balances) = state.validators_and_balances_mut();
    for (index, validator) in validators.iter_mut().enumerate() {
        let balance = balances
            .get(index)
            .copied()
            .ok_or(BeaconStateError::BalancesOutOfBounds(index))? as u128;

        if (balance).safe_add(downward_threshold)? < validator.effective_balance as u128
            || (validator.effective_balance as u128).safe_add(upward_threshold)? < balance
        {
            validator.effective_balance = std::cmp::min(
                balance.safe_sub(balance.safe_rem(spec.effective_balance_increment as u128)?)? as u64,
                spec.max_effective_balance,
            );
        }
    }
    Ok(())
}
