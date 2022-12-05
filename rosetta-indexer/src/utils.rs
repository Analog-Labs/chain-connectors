use rosetta_client::Client;
use rosetta_types::{AccountIdentifier, BlockTransaction, TransactionIdentifier, BlockRequest, network_identifier, NetworkIdentifier, PartialBlockIdentifier};
use subxt::{Config, OnlineClient, rpc::BlockNumber};

#[derive(Debug)]
pub enum Error {
    BlockNotFound,
    TransactionNotFound,
    NotSupported,
}

pub struct TxIndexerProps {
    pub max_block: i64,
    pub transaction_identifier: Option<TransactionIdentifier>,
    pub account_identifier: Option<AccountIdentifier>,
    pub status: Option<String>,
    pub operation_type: Option<String>,
    pub address: Option<String>,
    pub success: Option<bool>,
}

pub async fn get_indexed_transactions<T>(
    // state: &State,
    server_client: &Client,
    client: &OnlineClient<T>,
    network_identifier: NetworkIdentifier,
    req: TxIndexerProps,
) -> Result<Vec<BlockTransaction>, Error>
where
    T: Config,
{
    // let mut filtered_ex = vec![];
    for value in (0..req.max_block).rev() {
        
        //request block data from client
        let block_request = BlockRequest{
            network_identifier: network_identifier.clone(),
            block_identifier: PartialBlockIdentifier{
                index: Some(value as u64),
                hash: None,
            },
        };
        let data = server_client.block(&block_request).await;
        println!("data {:?}", data);
        println!("abc");


    }

    Ok(vec![])
    //     let block_hash = match client
    //         .rpc()
    //         .block_hash(Some(BlockNumber::from(value as u64)))
    //         .await
    //     {
    //         Ok(block_hash) => block_hash.ok_or(Error::BlockNotFound)?,
    //         Err(_) => return Err(Error::StorageFetch),
    //     };

    //     let block_data = match state.client.rpc().block(Some(block_hash)).await {
    //         Ok(block_data) => block_data.ok_or(Error::BlockNotFound)?,
    //         Err(_) => return Err(Error::StorageFetch),
    //     };

    //     let extrinsics = block_data.block.extrinsics;
    //     let (tx_data, can_fetch_data) = filter_extrinsic(
    //         &state.client,
    //         &state.ss58_address_format,
    //         &state.currency,
    //         extrinsics,
    //         &req,
    //         block_hash,
    //     )
    //     .await?;

    //     if !tx_data.is_empty() {
    //         for tx in tx_data {
    //             let block_transaction = BlockTransaction {
    //                 block_identifier: BlockIdentifier {
    //                     index: value as u64,
    //                     hash: format!("{:?}", block_hash),
    //                 },
    //                 transaction: tx,
    //             };

    //             filtered_ex.push(block_transaction);
    //         }
    //     }

    //     if !can_fetch_data {
    //         break;
    //     }
    // }
    // Ok(filtered_ex)
}

// pub async fn filter_extrinsic<T>(
//     client: &OnlineClient<T>,
//     address_format: &Ss58AddressFormat,
//     currency: &Currency,
//     extrinsics: Vec<OpaqueExtrinsic>,
//     req: &TxIndexerProps,
//     block_hash: T::Hash,
// ) -> Result<(Vec<Transaction>, bool), Error>
// where
//     T: Config,
// {
//     let mut can_fetch = true;
//     let mut vec_of_extrinsics = vec![];

//     for (ex_index, extrinsic) in extrinsics.iter().enumerate() {
//         let encoded_extrinsic = extrinsic.encode();
//         let tx_hash_hex = hex::encode(encoded_extrinsic);
//         let tx_hash = convert_extrinsic_to_hash(tx_hash_hex)?;
//         let mut vector_of_operations: Vec<Operation> = vec![];

//         let transaction_identifier = TransactionIdentifier {
//             hash: tx_hash.clone(),
//         };

//         if let Some(ref tx_id) = req.transaction_identifier {
//             if tx_id.hash.eq(&tx_hash) {
//                 continue;
//             }
//         }

//         let current_block_events = match get_block_events(client, block_hash).await {
//             Ok(events) => events,
//             Err(_) => {
//                 //can not get anymore events break the loop
//                 can_fetch = false;
//                 break;
//             }
//         };

//         let current_tx_events = current_block_events
//             .iter()
//             .filter(|event| {
//                 event.as_ref().unwrap().phase() == Phase::ApplyExtrinsic(ex_index as u32)
//             })
//             .collect::<Vec<_>>();

//         let extrinsic_status = match current_tx_events.last() {
//             Some(event) => event.as_ref().unwrap(),
//             None => continue,
//         };

//         if let Some(success_status) = req.success {
//             if success_status {
//                 let extrinsic_event_name = extrinsic_status.event_metadata().event();
//                 if !extrinsic_event_name.eq("ExtrinsicSuccess") {
//                     continue;
//                 }
//             } else {
//                 let extrinsic_event_name = extrinsic_status.event_metadata().event();
//                 if !extrinsic_event_name.eq("ExtrinsicFailed") {
//                     continue;
//                 }
//             }
//         }

//         for (event_index, event_data) in current_tx_events.iter().enumerate() {
//             let operation_identifier = OperationIdentifier {
//                 index: event_index as i64,
//                 network_index: None,
//             };

//             let event = event_data.as_ref().unwrap();
//             let event_name = event.event_metadata().event();

//             if let Some(operation_type) = req.operation_type.as_ref() {
//                 if !operation_type.eq(&event_name) {
//                     continue;
//                 }
//             }

//             let event_parsed = get_operation_data(event.clone(), address_format).unwrap();

//             if let Some(acc_identifier) = req.account_identifier.as_ref() {
//                 let mut event_addresses = vec![];
//                 if let Some(from) = event_parsed.from.clone() {
//                     event_addresses.push(from);
//                 }

//                 if let Some(to) = event_parsed.to.clone() {
//                     event_addresses.push(to);
//                 }

//                 let matched_address = if event_addresses.iter().any(|address| {
//                     address
//                         .to_owned()
//                         .eq(&acc_identifier.address.trim_start_matches("0x"))
//                 }) {
//                     true
//                 } else {
//                     match acc_identifier.sub_account.as_ref() {
//                         Some(sub_address) => event_addresses.iter().any(|address| {
//                             address
//                                 .to_owned()
//                                 .eq(&sub_address.address.trim_start_matches("0x"))
//                         }),
//                         None => false,
//                     }
//                 };

//                 if !matched_address {
//                     continue;
//                 }
//             }

//             //all checks passed process the operation
//             let event_metadata = event.event_metadata();

//             let mut vec_metadata = vec![];
//             for event in event_metadata.fields().iter() {
//                 let name = event.name();
//                 let type_name = event.type_name();
//                 vec_metadata.push(json!({"name":name, "type": type_name}));
//             }
//             let op_metadata = Value::Array(vec_metadata);

//             let op_account: Option<AccountIdentifier> = match event_parsed.from {
//                 Some(from) => match event_parsed.to {
//                     Some(to) => Some(AccountIdentifier {
//                         address: from,
//                         sub_account: Some(SubAccountIdentifier {
//                             address: to,
//                             metadata: None,
//                         }),
//                         metadata: None,
//                     }),
//                     None => Some(AccountIdentifier {
//                         address: from,
//                         sub_account: None,
//                         metadata: None,
//                     }),
//                 },
//                 None => None,
//             };

//             let op_amount: Option<Amount> = event_parsed.amount.map(|amount| Amount {
//                 value: amount,
//                 currency: currency.clone(),
//                 metadata: None,
//             });

//             let operation = Operation {
//                 operation_identifier,
//                 related_operations: None,
//                 r#type: event_parsed.event_type,
//                 status: None,
//                 account: op_account,
//                 amount: op_amount,
//                 coin_change: None,
//                 metadata: Some(op_metadata),
//             };

//             vector_of_operations.push(operation);
//         }

//         if !vector_of_operations.is_empty() {
//             let transaction = Transaction {
//                 transaction_identifier,
//                 operations: vector_of_operations,
//                 related_transactions: None,
//                 metadata: None,
//             };
//             vec_of_extrinsics.push(transaction);
//         }
//     }
//     Ok((vec_of_extrinsics, can_fetch))
// }
