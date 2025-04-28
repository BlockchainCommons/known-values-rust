use std::sync::{ Once, Mutex };
use paste::paste;

use super::KnownValuesStore;

/// A macro that declares a known value at compile time.
///
/// This macro creates two constants:
/// - A raw u64 value constant with the suffix `_RAW`
/// - A KnownValue constant with the given name and value
///
/// This is used internally to define all the standard Known Values in the registry.
///
/// # Examples
///
/// ```
/// use known_values::*;
/// use paste::paste;
///
/// // Define a custom known value
/// const_known_value!(1000, MY_CUSTOM_VALUE, "myCustomValue");
///
/// // Now MY_CUSTOM_VALUE is a constant KnownValue
/// assert_eq!(MY_CUSTOM_VALUE.value(), 1000);
/// assert_eq!(MY_CUSTOM_VALUE.name(), "myCustomValue");
///
/// paste! {
///     // MY_CUSTOM_VALUE_RAW is the raw u64 value
///     assert_eq!([<MY_CUSTOM_VALUE _RAW>], 1000);
/// }
/// ```
#[macro_export]
macro_rules! const_known_value {
    ($value:expr, $const_name:ident, $name:expr) => {
        paste! {
            pub const [<$const_name _RAW>]: u64 = $value;
        }
        pub const $const_name: $crate::KnownValue = $crate::KnownValue::new_with_static_name($value, $name);
    };
}

// For definitions see: https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-002-known-value.md#appendix-a-registry

//
// General
//

// 0 *unassigned*
const_known_value!(1, IS_A, "isA");
const_known_value!(2, ID, "id");
const_known_value!(3, SIGNED, "signed");
const_known_value!(4, NOTE, "note");
const_known_value!(5, HAS_RECIPIENT, "hasRecipient");
const_known_value!(6, SSKR_SHARE, "sskrShare");
const_known_value!(7, CONTROLLER, "controller");
const_known_value!(8, KEY, "key");
const_known_value!(9, DEREFERENCE_VIA, "dereferenceVia");
const_known_value!(10, ENTITY, "entity");
const_known_value!(11, NAME, "name");
const_known_value!(12, LANGUAGE, "language");
const_known_value!(13, ISSUER, "issuer");
const_known_value!(14, HOLDER, "holder");
const_known_value!(15, SALT, "salt");
const_known_value!(16, DATE, "date");
const_known_value!(17, UNKNOWN_VALUE, "Unknown");
const_known_value!(18, VERSION_VALUE, "version");
const_known_value!(19, HAS_SECRET, "hasSecret");
const_known_value!(20, DIFF_EDITS, "edits");
const_known_value!(21, VALID_FROM, "validFrom");
const_known_value!(22, VALID_UNTIL, "validUntil");
// 23-49 *unassigned*

//
// Attachments
//

const_known_value!(50, ATTACHMENT, "attachment");
const_known_value!(51, VENDOR, "vendor");
const_known_value!(52, CONFORMS_TO, "conformsTo");
// 53-59 *unassigned*

//
// XID Documents
//

const_known_value!(60, ALLOW, "allow");
const_known_value!(61, DENY, "deny");
const_known_value!(62, ENDPOINT, "endpoint");
const_known_value!(63, DELEGATE, "delegate");
const_known_value!(64, PROVENANCE, "provenance");
const_known_value!(65, PRIVATE_KEY, "privateKey");
const_known_value!(66, SERVICE, "service");
const_known_value!(67, CAPABILITY, "capability");
// 68-69 *unassigned*

//
// XID Privileges
//

const_known_value!(70, PRIVILEGE_ALL, "All");
const_known_value!(71, PRIVILEGE_AUTH, "Auth");
const_known_value!(72, PRIVILEGE_SIGN, "Sign");
const_known_value!(73, PRIVILEGE_ENCRYPT, "Encrypt");
const_known_value!(74, PRIVILEGE_ELIDE, "Elide");
const_known_value!(75, PRIVILEGE_ISSUE, "Issue");
const_known_value!(76, PRIVILEGE_ACCESS, "Access");
// 77-79 *unassigned*
const_known_value!(80, PRIVILEGE_DELEGATE, "Delegate");
const_known_value!(81, PRIVILEGE_VERIFY, "Verify");
const_known_value!(82, PRIVILEGE_UPDATE, "Update");
const_known_value!(83, PRIVILEGE_TRANSFER, "Transfer");
const_known_value!(84, PRIVILEGE_ELECT, "Elect");
const_known_value!(85, PRIVILEGE_BURN, "Burn");
const_known_value!(86, PRIVILEGE_REVOKE, "Revoke");
// 87-99 *unassigned*

//
// Expression and Function Calls
//

const_known_value!(100, BODY, "body");
const_known_value!(101, RESULT, "result");
const_known_value!(102, ERROR, "error");
const_known_value!(103, OK_VALUE, "OK");
const_known_value!(104, PROCESSING_VALUE, "Processing");
const_known_value!(105, SENDER, "sender");
const_known_value!(106, SENDER_CONTINUATION, "senderContinuation");
const_known_value!(107, RECIPIENT_CONTINUATION, "recipientContinuation");
const_known_value!(108, CONTENT, "content");
// 109-199 *unassigned*

//
// Cryptography
//

const_known_value!(200, SEED_TYPE, "Seed");
const_known_value!(201, PRIVATE_KEY_TYPE, "PrivateKey");
const_known_value!(202, PUBLIC_KEY_TYPE, "PublicKey");
const_known_value!(203, MASTER_KEY_TYPE, "MasterKey");
// 204-299 *unassigned*

//
// Cryptocurrency Assets
//

const_known_value!(300, ASSET, "asset");
const_known_value!(301, BITCOIN_VALUE, "BTC");
const_known_value!(302, ETHEREUM_VALUE, "ETH");
const_known_value!(303, TEZOS_VALUE, "XTZ");
// 304-399 *unassigned*

//
// Cryptocurrency Networks
//

const_known_value!(400, NETWORK, "network");
const_known_value!(401, MAIN_NET_VALUE, "MainNet");
const_known_value!(402, TEST_NET_VALUE, "TestNet");
// 403-499 *unassigned*

//
// Bitcoin
//

const_known_value!(500, BIP32_KEY_TYPE, "BIP32Key");
const_known_value!(501, CHAIN_CODE, "chainCode");
const_known_value!(502, DERIVATION_PATH_TYPE, "DerivationPath");
const_known_value!(503, PARENT_PATH, "parent");
const_known_value!(504, CHILDREN_PATH, "children");
const_known_value!(505, PARENT_FINGERPRINT, "parentFingerprint");
const_known_value!(506, PSBT_TYPE, "PSBT");
const_known_value!(507, OUTPUT_DESCRIPTOR_TYPE, "OutputDescriptor");
const_known_value!(508, OUTPUT_DESCRIPTOR, "outputDescriptor");
// 509-599 *unassigned*

//
// Graphs
//

const_known_value!(600, GRAPH, "graph");
const_known_value!(601, SOURCE_TARGET_GRAPH, "SourceTargetGraph");
const_known_value!(602, PARENT_CHILD_GRAPH, "ParentChildGraph");
const_known_value!(603, DIGRAPH, "Digraph");
const_known_value!(604, ACYCLIC_GRAPH, "AcyclicGraph");
const_known_value!(605, MULTIGRAPH, "Multigraph");
const_known_value!(606, PSEUDOGRAPH, "Pseudograph");
const_known_value!(607, GRAPH_FRAGMENT, "GraphFragment");
const_known_value!(608, DAG, "DAG");
const_known_value!(609, TREE, "Tree");
const_known_value!(610, FOREST, "Forest");
const_known_value!(611, COMPOUND_GRAPH, "CompoundGraph");
const_known_value!(612, HYPERGRAPH, "Hypergraph");
const_known_value!(613, DIHYPERGRAPH, "Dihypergraph");
// 614-699 *unassigned*
const_known_value!(700, NODE, "node");
const_known_value!(701, EDGE, "edge");
const_known_value!(702, SOURCE, "source");
const_known_value!(703, TARGET, "target");
const_known_value!(704, PARENT, "parent");
const_known_value!(705, CHILD, "child");
// 706-... *unassigned*

/// A lazily initialized singleton that holds the global registry of known values.
///
/// This type provides thread-safe, lazy initialization of the global KnownValuesStore
/// that contains all the predefined Known Values in the registry. The store is created
/// only when first accessed, and subsequent accesses reuse the same instance.
///
/// This is used internally by the crate and should not typically be needed by users
/// of the API, who should access Known Values through the constants exposed in the
/// `known_values` module.
///
/// # Thread Safety
///
/// The implementation uses a mutex to protect the store, and initialization is
/// performed only once across all threads using `std::sync::Once`.
#[doc(hidden)]
#[derive(Debug)]
pub struct LazyKnownValues {
    init: Once,
    data: Mutex<Option<KnownValuesStore>>,
}

impl LazyKnownValues {
    /// Gets the global KnownValuesStore, initializing it if necessary.
    ///
    /// This method guarantees that initialization occurs exactly once,
    /// even when called from multiple threads simultaneously.
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownValuesStore>> {
        self.init.call_once(|| {
            let m = KnownValuesStore::new([
                IS_A,
                ID,
                SIGNED,
                NOTE,
                HAS_RECIPIENT,
                SSKR_SHARE,
                CONTROLLER,
                KEY,
                DEREFERENCE_VIA,
                ENTITY,
                NAME,
                LANGUAGE,
                ISSUER,
                HOLDER,
                SALT,
                DATE,
                UNKNOWN_VALUE,
                VERSION_VALUE,
                HAS_SECRET,
                DIFF_EDITS,
                VALID_FROM,
                VALID_UNTIL,

                ATTACHMENT,
                VENDOR,
                CONFORMS_TO,

                ALLOW,
                DENY,
                ENDPOINT,
                DELEGATE,
                PROVENANCE,
                PRIVATE_KEY,
                SERVICE,
                CAPABILITY,

                PRIVILEGE_ALL,
                PRIVILEGE_AUTH,
                PRIVILEGE_SIGN,
                PRIVILEGE_ENCRYPT,
                PRIVILEGE_ELIDE,
                PRIVILEGE_ISSUE,
                PRIVILEGE_ACCESS,

                PRIVILEGE_DELEGATE,
                PRIVILEGE_VERIFY,
                PRIVILEGE_UPDATE,
                PRIVILEGE_TRANSFER,
                PRIVILEGE_ELECT,
                PRIVILEGE_BURN,
                PRIVILEGE_REVOKE,

                BODY,
                RESULT,
                ERROR,
                OK_VALUE,
                PROCESSING_VALUE,
                SENDER,
                SENDER_CONTINUATION,
                RECIPIENT_CONTINUATION,
                CONTENT,

                SEED_TYPE,
                PRIVATE_KEY_TYPE,
                PUBLIC_KEY_TYPE,
                MASTER_KEY_TYPE,

                ASSET,
                BITCOIN_VALUE,
                ETHEREUM_VALUE,
                TEZOS_VALUE,

                NETWORK,
                MAIN_NET_VALUE,
                TEST_NET_VALUE,

                BIP32_KEY_TYPE,
                CHAIN_CODE,
                DERIVATION_PATH_TYPE,
                PARENT_PATH,
                CHILDREN_PATH,
                PARENT_FINGERPRINT,
                PSBT_TYPE,
                OUTPUT_DESCRIPTOR_TYPE,
                OUTPUT_DESCRIPTOR,

                GRAPH,
                SOURCE_TARGET_GRAPH,
                PARENT_CHILD_GRAPH,
                DIGRAPH,
                ACYCLIC_GRAPH,
                MULTIGRAPH,
                PSEUDOGRAPH,
                GRAPH_FRAGMENT,
                DAG,
                TREE,
                FOREST,
                COMPOUND_GRAPH,
                HYPERGRAPH,
                DIHYPERGRAPH,
                NODE,
                EDGE,
                SOURCE,
                TARGET,
                PARENT,
                CHILD,
            ]);
            *self.data.lock().unwrap() = Some(m);
        });
        self.data.lock().unwrap()
    }
}

/// The global registry of Known Values.
///
/// This static instance provides access to all standard Known Values defined in the
/// registry specification. It is lazily initialized on first access.
///
/// Most users should not need to interact with this directly, as the predefined
/// Known Values are exposed as constants in the `known_values` module.
///
/// # Examples
///
/// ```
/// use known_values::*;
///
/// // Access the global store
/// let binding = KNOWN_VALUES.get();
/// let known_values = binding.as_ref().unwrap();
///
/// // Look up a Known Value by name
/// let is_a = known_values.known_value_named("isA").unwrap();
/// assert_eq!(is_a.value(), 1);
/// ```
pub static KNOWN_VALUES: LazyKnownValues = LazyKnownValues {
    init: Once::new(),
    data: Mutex::new(None),
};

#[cfg(test)]
mod tests {
    #[test]
    fn test_1() {
        assert_eq!(crate::IS_A.value(), 1);
        assert_eq!(crate::IS_A.name(), "isA");
        let binding = crate::KNOWN_VALUES.get();
        let known_values = binding.as_ref().unwrap();
        assert_eq!(known_values.known_value_named("isA").unwrap().value(), 1);
    }
}
