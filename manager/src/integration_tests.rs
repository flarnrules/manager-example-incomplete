#[cfg(test)]
mod tests {
    use crate::helpers::ManagerContract;
    use crate::msg::{ExecuteMsg, GetContractsResponse, InstantiateMsg, QueryMsg};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use counter::{self, GetCountResponse};

    pub fn contract_manager() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn contract_counter() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            counter::contract::execute,
            counter::contract::instantiate,
            counter::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64) {
        let mut app = mock_app();
        let manager_id = app.store_code(contract_manager());
        let counter_id = app.store_code(contract_counter());
        (app, manager_id, counter_id)
    }

    fn manager_instantiate(app: &mut App, manager_id: u64) -> ManagerContract {
        let msg = InstantiateMsg {};
        let manager_contract_address = app
            .instantiate_contract(
                manager_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "counter manager",
                None,
            )
            .unwrap();
        ManagerContract(manager_contract_address)
    }

    //call InstantiateNew on the manager contract to create a new counter instance.
    fn instantiate_new(app: &mut App, manager_contract: &ManagerContract, counter_id: u64) {
        let msg = ExecuteMsg::InstantiateNewCounter {
            code_id: counter_id,
        };
        let cosmos_msg = manager_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    //increment the counter from the manager contract
    fn increment(app: &mut App, manager_contract: &ManagerContract, contract_addr: String) {
        let msg = ExecuteMsg::Increment {
            contract: contract_addr,
        };
        let cosmos_msg = manager_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    //reset the counter from the manager contract
    fn reset(app: &mut App, manager_contract: &ManagerContract, contract_addr: String, count: i32) {
        let msg = ExecuteMsg::Reset {
            contract: contract_addr,
            count,
        };
        let cosmos_msg = manager_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
    }

    fn get_contracts(app: &App, manager_contract: &ManagerContract) -> GetContractsResponse {
        app.wrap()
            .query_wasm_smart(manager_contract.addr(), &QueryMsg::GetContracts {})
            .unwrap()
    }

    fn get_count(app: &App, contract_addr: &str) -> GetCountResponse {
        app.wrap()
            .query_wasm_smart(contract_addr, &counter::QueryMsg::GetCount {})
            .unwrap()
    }

    #[test]
    fn create_one_counter() {
        let (mut app, manager_id, counter_id) = store_code();
        let manager_contract = manager_instantiate(&mut app, manager_id);

        instantiate_new(&mut app, &manager_contract, counter_id);
        let res = get_contracts(&app, &manager_contract);

        assert_eq!(res.contracts.len(), 1);
        assert_eq!(res.contracts[0].1.address, "contract1");

        //println!("{:?}", res.contracts[0].1);
    }

    #[test]
    fn create_two_counters() {
        let (mut app, manager_id, counter_id) = store_code();
        let manager_contract = manager_instantiate(&mut app, manager_id);

        instantiate_new(&mut app, &manager_contract, counter_id);
        instantiate_new(&mut app, &manager_contract, counter_id);

        let res = get_contracts(&app, &manager_contract);

        assert_eq!(res.contracts.len(), 2);
        assert_eq!(res.contracts[0].1.address, "contract1");
        assert_eq!(res.contracts[1].1.address, "contract2");
    }

    #[test]
    fn create_counter_and_increment() {
        let (mut app, manager_id, counter_id) = store_code();
        let manager_contract = manager_instantiate(&mut app, manager_id);

        instantiate_new(&mut app, &manager_contract, counter_id);
        increment(&mut app, &manager_contract, "contract1".to_string());

        let res = get_contracts(&app, &manager_contract);
        let res = get_count(&app, res.contracts[0].1.address.as_str());
        assert_eq!(res.count, 1);
    }

    #[test]
    fn create_counter_and_increment_twice() {
        unimplemented!()
    }

    #[test]
    fn create_counter_and_increment_and_reset() {
        unimplemented!()
    }

    #[test]
    fn create_two_counters_and_increment_each() {
        unimplemented!()
    }
}
