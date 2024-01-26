use derivative::Derivative;

use super::{
    access_list::AccessList, signature::Signature, GasPrice, SignedTransactionT, TransactionT,
};
#[cfg(feature = "with-crypto")]
use crate::crypto::DefaultCrypto;
#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction};
use crate::{
    bytes::Bytes,
    crypto::Crypto,
    eth_hash::{Address, TxHash, H256},
    eth_uint::U256,
};

#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo),
    codec(dumb_trait_bound)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Derivative)]
#[derivative(Clone, PartialEq, Eq, Debug)]
pub struct SignedTransaction<T> {
    #[cfg_attr(feature = "serde", serde(rename = "hash"))]
    pub tx_hash: TxHash,
    #[cfg_attr(
        feature = "serde",
        serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned", flatten)
    )]
    pub payload: T,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub signature: Signature,
}

// impl <T> Default for SignedTransaction<T> where T: Default {
//     fn default() -> Self {
//         Self {
//             tx_hash: H256::zero(),
//             payload: T::default(),
//             signature: Signature::default(),
//         }
//     }
// }

impl<T> SignedTransaction<T>
where
    T: TransactionT,
{
    pub fn new(payload: T, signature: Signature) -> Self {
        let tx_hash = payload.compute_tx_hash(&signature);
        Self { tx_hash, payload, signature }
    }

    /// Recovery the signer address
    /// # Errors
    /// Returns an error if the signature is invalid
    pub fn compute_from<C: Crypto>(&self) -> Result<Address, C::Error> {
        let sighash = self.payload.sighash();
        C::secp256k1_ecdsa_recover(&self.signature, sighash)
    }

    /// Recovery the signer address
    /// # Errors
    /// Returns an error if the signature is invalid
    #[cfg(feature = "with-crypto")]
    pub fn from(&self) -> Result<Address, <DefaultCrypto as Crypto>::Error> {
        self.compute_from::<DefaultCrypto>()
    }
}

#[cfg(feature = "with-rlp")]
impl<T> RlpEncodableTransaction for SignedTransaction<T>
where
    T: RlpEncodableTransaction + TransactionT,
{
    fn rlp_append(&self, s: &mut rlp::RlpStream, signature: Option<&Signature>) {
        <T as RlpEncodableTransaction>::rlp_append(&self.payload, s, signature);
    }
}

#[cfg(feature = "with-rlp")]
impl<T> rlp::Encodable for SignedTransaction<T>
where
    T: RlpEncodableTransaction + TransactionT,
{
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        <T as RlpEncodableTransaction>::rlp_append(&self.payload, s, Some(&self.signature));
    }
}

#[cfg(feature = "with-rlp")]
impl<T> RlpDecodableTransaction for SignedTransaction<T>
where
    T: RlpDecodableTransaction + TransactionT,
{
    // For SignedTransaction we always decode the signature
    fn rlp_decode_unsigned(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let (payload, signature) = <T as RlpDecodableTransaction>::rlp_decode(rlp, true)?;
        let signature = signature.ok_or(rlp::DecoderError::Custom("tx signature is missing"))?;
        let tx_hash = payload.compute_tx_hash(&signature);
        Ok(Self { tx_hash, payload, signature })
    }

    fn rlp_decode(
        rlp: &rlp::Rlp,
        _decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        let signed_tx = <Self as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)?;
        let signature = signed_tx.signature;
        Ok((signed_tx, Some(signature)))
    }
}

#[cfg(feature = "with-rlp")]
impl<T> rlp::Decodable for SignedTransaction<T>
where
    T: RlpDecodableTransaction + TransactionT,
{
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let (payload, signature) = <T as RlpDecodableTransaction>::rlp_decode(rlp, true)?;
        let signature = signature.ok_or(rlp::DecoderError::Custom("tx signature is missing"))?;
        let tx_hash = payload.compute_tx_hash(&signature);
        Ok(Self { tx_hash, payload, signature })
    }
}

impl<T> TransactionT for SignedTransaction<T>
where
    T: TransactionT,
{
    type ExtraFields = <T as TransactionT>::ExtraFields;

    // Compute the tx-hash using the provided signature
    fn compute_tx_hash(&self, signature: &Signature) -> H256 {
        self.payload.compute_tx_hash(signature)
    }
    fn chain_id(&self) -> Option<u64> {
        self.payload.chain_id()
    }
    fn nonce(&self) -> u64 {
        self.payload.nonce()
    }
    fn gas_price(&self) -> GasPrice {
        self.payload.gas_price()
    }
    fn gas_limit(&self) -> u64 {
        self.payload.gas_limit()
    }
    fn to(&self) -> Option<Address> {
        self.payload.to()
    }
    fn value(&self) -> U256 {
        self.payload.value()
    }
    fn data(&self) -> &[u8] {
        self.payload.data()
    }
    fn sighash(&self) -> H256 {
        self.payload.sighash()
    }
    fn access_list(&self) -> Option<&AccessList> {
        self.payload.access_list()
    }
    fn transaction_type(&self) -> Option<u8> {
        self.payload.transaction_type()
    }
    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        self.payload.extra_fields()
    }
    fn encode(&self, signature: Option<&Signature>) -> Bytes {
        self.payload.encode(signature)
    }
}

impl<T> SignedTransactionT for SignedTransaction<T>
where
    T: TransactionT,
{
    fn tx_hash(&self) -> H256 {
        self.tx_hash
    }

    fn signature(&self) -> Signature {
        self.signature
    }

    fn encode_signed(&self) -> Bytes {
        TransactionT::encode(self, Some(&self.signature))
    }
}

#[cfg(all(test, feature = "serde", feature = "with-rlp", feature = "with-crypto"))]
mod tests {
    use super::super::eip2930::Eip2930Transaction;
    use crate::{
        bytes::Bytes,
        eth_hash::{Address, H256},
        rpc::RpcTransaction,
        transactions::{
            access_list::{AccessList, AccessListItem},
            signature::{RecoveryId, Signature},
            SignedTransactionT,
        },
        TypedTransaction,
    };
    use hex_literal::hex;

    fn build_eip2930() -> (Eip2930Transaction, Signature) {
        let tx = Eip2930Transaction {
            chain_id: 1,
            nonce: 117,
            gas_price: 28_379_509_371u128.into(),
            gas_limit: 187_293,
            to: Some(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").into()),
            value: 3_650_000_000_000_000_000u128.into(),
            data: Bytes::from(hex!("3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000")),
            access_list: AccessList(vec![AccessListItem {
                address: Address::from(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad")),
                storage_keys: vec![
                    H256::zero(),
                    H256::from(hex!(
                        "a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6"
                    )),
                    H256::from(hex!(
                        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    )),
                ],
            }]),
        };
        let signature = Signature {
            v: RecoveryId::new(0x01),
            r: hex!("5fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9").into(),
            s: hex!("41425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1").into(),
        };
        (tx, signature)
    }

    #[test]
    fn serde_encode_works() {
        use crate::SignedTransaction;

        let (tx, sig) = build_eip2930();
        let signed_tx = super::SignedTransaction::new(tx, sig);
        let actual = serde_json::to_value(&signed_tx).unwrap();
        let expected = serde_json::json!({
            "hash": "0xa777326ad77731344d00263b06843be6ef05cbe9ab699e2ed0d1448f8b2b50a3",
            "chainId": "0x1",
            "nonce": "0x75",
            "gasPrice": "0x69b8cf27b",
            "gas": "0x2db9d",
            "to": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
            "value": "0x32a767a9562d0000",
            "data": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000",
            "accessList": [
                {
                    "address": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
                    "storageKeys": [
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "0xa19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6",
                        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    ]
                }
            ],
            "v": "0x1",
            "r": "0x5fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9",
            "s": "0x41425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1"
        });
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&signed_tx).unwrap();
        let decoded =
            serde_json::from_str::<SignedTransaction<Eip2930Transaction>>(&json_str).unwrap();
        assert_eq!(signed_tx, decoded);
    }

    #[test]
    fn serde_decode_works() {
        use crate::SignedTransaction;
        let json_tx = r#"
        {
            "hash": "0xb3fbbda7862791ec65c07b1162bd6c6aa10efc89196a8727790a9b03b3ca7bab",
            "nonce": "0x115",
            "blockHash": "0x533ae98e36b11720a6095de0cbae802e80719cede1e3a65e02379436993a2007",
            "blockNumber": "0x37cd6",
            "transactionIndex": "0x0",
            "from": "0xcf684dfb8304729355b58315e8019b1aa2ad1bac",
            "to": null,
            "value": "0x0",
            "gasPrice": "0xba43b7400",
            "gas": "0x2f4d60",
            "input": "0x60606040526009600060146101000a81548160ff021916908302179055505b6000600033600060006101000a81548173ffffffffffffffffffffffffffffffffffffffff02191690830217905550600091505b600060149054906101000a900460ff1660ff168260ff16101561010457600090505b600060149054906101000a900460ff1660ff168160ff1610156100f6578082600060149054906101000a900460ff1602016001600050826009811015610002579090601202016000508360098110156100025790906002020160005060010160146101000a81548160ff021916908302179055505b8080600101915050610074565b5b8180600101925050610052565b5b5050610160806101166000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900480634166c1fd1461004457806341c0e1b51461007457610042565b005b61005b600480359060200180359060200150610081565b604051808260ff16815260200191505060405180910390f35b61007f6004506100cc565b005b60006001600050836009811015610002579090601202016000508260098110156100025790906002020160005060010160149054906101000a900460ff1690506100c6565b92915050565b600060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16141561015d57600060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16ff5b5b56",
            "chainId": "0x1",
            "v": "0x1b",
            "r": "0x834b0e7866457890809cb81a33a59380e890e1cc0d6e17a81382e99132b16bc8",
            "s": "0x65dcc7686efc8f7937b3ae0d09d682cd3a7ead281a920ec39d4e2b0c34e972be",
            "type": "0x0"
        }"#;

        let mut tx = serde_json::from_str::<RpcTransaction>(json_tx).unwrap();
        tx.chain_id = None;
        let tx = SignedTransaction::<TypedTransaction>::try_from(tx).unwrap();

        let expected = hex!("f902cb820115850ba43b7400832f4d608080b9027660606040526009600060146101000a81548160ff021916908302179055505b6000600033600060006101000a81548173ffffffffffffffffffffffffffffffffffffffff02191690830217905550600091505b600060149054906101000a900460ff1660ff168260ff16101561010457600090505b600060149054906101000a900460ff1660ff168160ff1610156100f6578082600060149054906101000a900460ff1602016001600050826009811015610002579090601202016000508360098110156100025790906002020160005060010160146101000a81548160ff021916908302179055505b8080600101915050610074565b5b8180600101925050610052565b5b5050610160806101166000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900480634166c1fd1461004457806341c0e1b51461007457610042565b005b61005b600480359060200180359060200150610081565b604051808260ff16815260200191505060405180910390f35b61007f6004506100cc565b005b60006001600050836009811015610002579090601202016000508260098110156100025790906002020160005060010160149054906101000a900460ff1690506100c6565b92915050565b600060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16141561015d57600060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16ff5b5b561ba0834b0e7866457890809cb81a33a59380e890e1cc0d6e17a81382e99132b16bc8a065dcc7686efc8f7937b3ae0d09d682cd3a7ead281a920ec39d4e2b0c34e972be");
        let actual = tx.encode_signed();
        assert_eq!(actual, Bytes::from(&expected));
        let actual = tx.from().unwrap();
        let expected = Address::from(hex!("cf684dfb8304729355b58315e8019b1aa2ad1bac"));
        assert_eq!(actual, expected);
    }
}
