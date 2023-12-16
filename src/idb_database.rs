//! Database-related code

use fancy_constructor::new;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DomException, IdbTransactionMode};

pub(crate) use idb_version_change_event::IdbVersionChangeCallback;
pub use idb_version_change_event::IdbVersionChangeEvent;

use crate::dom_string_iterator::DomStringIterator;
use crate::idb_object_store::{IdbObjectStore, IdbObjectStoreParameters};
use crate::idb_transaction::IdbTransaction;
use crate::internal_utils::arrayify_slice;
use crate::request::{OpenDbRequest, VoidOpenDbRequest};

mod idb_version_change_event;

/// Wrapper for an [`IndexedDB`](web_sys::IdbDatabase)
#[derive(Debug, new, Clone)]
#[new(vis(pub(crate)))]
pub struct IdbDatabase {
    inner: web_sys::IdbDatabase,

    #[new(default)]
    on_version_change: Option<IdbVersionChangeCallback>,
}

type OpenDbResult = Result<OpenDbRequest, DomException>;

impl IdbDatabase {
    /// Open the database with the given name
    pub fn open(name: &str) -> OpenDbResult {
        Ok(OpenDbRequest::new(factory().open(name)?))
    }

    /// Open the database with the given name and u32 version
    pub fn open_u32(name: &str, version: u32) -> OpenDbResult {
        Ok(OpenDbRequest::new(factory().open_with_u32(name, version)?))
    }

    /// Open the database with the given name and f64 version
    pub fn open_f64(name: &str, version: f64) -> OpenDbResult {
        Ok(OpenDbRequest::new(factory().open_with_f64(name, version)?))
    }

    /// List the names of the object stores within this database
    #[inline]
    pub fn object_store_names(&self) -> impl Iterator<Item = String> + 'static {
        DomStringIterator::from(self.inner.object_store_names())
    }

    /// Get the database name
    #[inline]
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get the database version
    #[inline]
    #[must_use]
    pub fn version(&self) -> f64 {
        self.inner.version()
    }

    /// Close the database connection
    #[inline]
    pub fn close(&self) {
        self.inner.close();
    }

    /// Delete the object store with the given name
    #[inline]
    pub fn delete_object_store(&self, name: &str) -> Result<(), DomException> {
        Ok(self.inner.delete_object_store(name)?)
    }

    /// Close and delete the database
    pub fn delete(self) -> Result<VoidOpenDbRequest, DomException> {
        let name = self.name();
        self.close();
        Self::delete_by_name(&name)
    }

    /// Delete the database with the given name
    pub fn delete_by_name(name: &str) -> Result<VoidOpenDbRequest, DomException> {
        Ok(VoidOpenDbRequest::new(factory().delete_database(name)?))
    }

    /// Set the callback to execute when the versionchange event is fired
    pub fn set_on_version_change<F>(&mut self, callback: Option<F>)
    where
        F: Fn(&IdbVersionChangeEvent) -> Result<(), JsValue> + 'static,
    {
        self.on_version_change = if let Some(callback) = callback {
            let cb = IdbVersionChangeEvent::wrap_callback(callback);
            self.inner
                .set_onversionchange(Some(cb.as_ref().unchecked_ref()));
            Some(cb)
        } else {
            self.inner.set_onversionchange(None);
            None
        };
    }

    /// Start a transaction on the given object store
    pub fn transaction_on_one(&self, name: &str) -> Result<IdbTransaction, DomException> {
        let inner = self.inner.transaction_with_str(name)?;
        Ok(IdbTransaction::new(inner, self))
    }

    /// Start a transaction on the given object stores
    #[inline]
    pub fn transaction_on_multi(&self, names: &[&str]) -> Result<IdbTransaction, DomException> {
        self.transaction_on_multi_with_array(&arrayify_slice(names))
    }

    /// Start a transaction on the given JS array of object store names
    pub fn transaction_on_multi_with_array<V: JsCast>(
        &self,
        names: &V,
    ) -> Result<IdbTransaction, DomException> {
        let res = self
            .inner
            .transaction_with_str_sequence(names.unchecked_ref())?;
        Ok(IdbTransaction::new(res, self))
    }

    /// Start a transaction on the given object store with the given mode
    pub fn transaction_on_one_with_mode(
        &self,
        name: &str,
        mode: IdbTransactionMode,
    ) -> Result<IdbTransaction, DomException> {
        let res = self.inner.transaction_with_str_and_mode(name, mode)?;
        Ok(IdbTransaction::new(res, self))
    }

    /// Start a transaction on the given object stores with the given mode
    #[inline]
    pub fn transaction_on_multi_with_mode(
        &self,
        names: &[&str],
        mode: IdbTransactionMode,
    ) -> Result<IdbTransaction, DomException> {
        self.transaction_on_multi_with_mode_and_array(&arrayify_slice(names), mode)
    }

    /// Start a transaction on the given JS array of object store names with the given mode
    pub fn transaction_on_multi_with_mode_and_array<V: JsCast>(
        &self,
        names: &V,
        mode: IdbTransactionMode,
    ) -> Result<IdbTransaction, DomException> {
        let res = self
            .inner
            .transaction_with_str_sequence_and_mode(names.unchecked_ref(), mode)?;
        Ok(IdbTransaction::new(res, self))
    }

    /// Create an object store with the given name
    pub fn create_object_store(&self, name: &str) -> Result<IdbObjectStore, DomException> {
        let inner = self.inner.create_object_store(name)?;
        Ok(IdbObjectStore::from_db(inner, self))
    }

    /// Create an object store with the given name & optional parameters
    pub fn create_object_store_with_params(
        &self,
        name: &str,
        params: &IdbObjectStoreParameters,
    ) -> Result<IdbObjectStore, DomException> {
        let inner = self
            .inner
            .create_object_store_with_optional_parameters(name, params.as_js_value())?;
        Ok(IdbObjectStore::from_db(inner, self))
    }
}

impl Drop for IdbDatabase {
    fn drop(&mut self) {
        if self.on_version_change.is_some() {
            self.inner.set_onversionchange(None);
        }
    }
}

impl_display_for_named!(IdbDatabase);

fn factory() -> web_sys::IdbFactory {
    #[wasm_bindgen]
    extern "C" {
        type Global;

        #[wasm_bindgen(method, getter, js_name = Window)]
        fn window(this: &Global) -> JsValue;

        #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
        fn worker(this: &Global) -> JsValue;

        #[wasm_bindgen(method, getter, js_name = global)]
        fn node_global(this: &Global) -> JsValue;
    }

    #[wasm_bindgen]
    extern "C" {
        type NodeGlobal;

        #[wasm_bindgen(method, getter, catch, js_name = indexedDB)]
        fn indexed_db(this: &NodeGlobal) -> Result<Option<web_sys::IdbFactory>, JsValue>;
    }

    let global: Global = js_sys::global().unchecked_into();

    if !global.window().is_undefined() {
        global
            .unchecked_into::<web_sys::Window>()
            .indexed_db()
            .expect("No `indexedDB` getter in `Window`")
            .expect("The `indexedDB` getter returned `null` or `undefined`")
    } else if !global.worker().is_undefined() {
        global
            .unchecked_into::<web_sys::WorkerGlobalScope>()
            .indexed_db()
            .expect("No `indexedDB` getter in `WorkerGlobalScope`")
            .expect("The `indexedDB` getter returned `null` or `undefined`")
    } else if !global.node_global().is_undefined() {
        global
            .unchecked_into::<NodeGlobal>()
            .indexed_db()
            .expect("No `indexedDB` getter in the Node.js `global` environment")
            .expect("The `indexedDB` getter returned `null` or `undefined`")
    } else {
        panic!(
            "Only supported in a browser, or web worker, or Node.js with a polyfill for IndexedDB"
        );
    }
}

#[cfg(test)]
pub mod test {
    use core::future::Future;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::request::IdbOpenDbRequestLike;
    use crate::{IdbKeyPath, IdbQuerySource};

    use super::*;

    fn db_name() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn open_db(req: OpenDbRequest) -> IdbDatabase {
        req.await.expect("Future failed")
    }

    fn open_db_req(req: Result<OpenDbRequest, DomException>) -> impl Future<Output = IdbDatabase> {
        open_db(req.expect("Base open failed"))
    }

    pub mod object_store_names {
        test_mod_init!();

        test_case!(async empty_iter => {
            let db = open_db_req(IdbDatabase::open(&db_name())).await;
            let stores: Vec<String> = db.object_store_names().collect();
            assert_eq!(stores, Vec::<String>::new());
        });

        test_case!(async iter_with_two => {
            fn on_upgrade_needed(evt: &IdbVersionChangeEvent) -> Result<(), JsValue> {
                evt.db().create_object_store("store1")?;
                evt.db().create_object_store("store2")?;
                let _ = evt.transaction(); // make sure it doesn't panic
                Ok(())
            }

            let mut req = IdbDatabase::open(&db_name()).expect("Base open");
            req.set_on_upgrade_needed(Some(on_upgrade_needed));
            let db = open_db(req).await;
            let stores: Vec<String> = db.object_store_names().collect();

            assert_eq!(stores, vec![String::from("store1"), String::from("store2")]);
        });
    }

    pub mod open {
        test_mod_init!();

        #[allow(clippy::needless_pass_by_value)]
        fn test_version(db: &IdbDatabase, version_expected: f64, name_expected: String) {
            assert_eq!(db.name(), name_expected, "name");
            assert!((db.version() - version_expected).abs() < 0.01, "version");
        }

        test_case!(async should_open_without_version => {
            let name = db_name();
            test_version(&open_db_req(IdbDatabase::open(&name)).await, 1.0, name);
        });

        test_case!(async should_open_with_u32 => {
            let name = db_name();
            test_version(&open_db_req(IdbDatabase::open_u32(&name, 101)).await, 101.0, name);
        });

        test_case!(async should_open_with_f64 => {
            let name = db_name();
            test_version(&open_db_req(IdbDatabase::open_f64(&name, 42.0)).await, 42.0, name);
        });
    }

    pub mod deletions {
        test_mod_init!();

        test_case!(async delete_object_store => {
            let db_name = db_name();

            let mut req = IdbDatabase::open_u32(&db_name, 1).expect("open 1");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().create_object_store("s1")?;
                evt.db().create_object_store("s2")?;
                Ok(())
            }));
            let db = req.await.expect("db await 1");
            db.close();

            let mut req = IdbDatabase::open_u32(&db_name, 2).expect("open 2");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().delete_object_store("s1")?;
                Ok(())
            }));
            let db = req.await.expect("db await 2");
            let stores: Vec<String> = db.object_store_names().collect();
            let exp = vec![String::from("s2"); 1];

            assert_eq!(stores, exp);
        });

        test_case!(async delete_by_name => {
            async fn do_open(name: &str, v: u32, calls: Rc<RefCell<u8>>) -> IdbDatabase {
                let mut req = IdbDatabase::open_u32(name, v).expect("open");
                req.set_on_upgrade_needed(Some(move |_: &IdbVersionChangeEvent| {
                    let curr = *calls.borrow();
                    calls.replace(curr + 1);
                    Ok(())
                }));
                req.await.expect("db await")
            }

            let db_name = db_name();
            let calls = Rc::new(RefCell::new(0));

            let db = do_open(&db_name, 1, calls.clone()).await;
            db.delete().expect("Delete call").await.expect("delete promise");
            do_open(&db_name, 1, calls.clone()).await;

            assert_eq!(*calls.borrow(), 2);
        });
    }

    pub mod tx_open {
        test_mod_init!();

        #[allow(clippy::needless_pass_by_value)]
        fn check_transaction(
            res: Result<IdbTransaction, DomException>,
            mode: IdbTransactionMode,
            exp: Vec<String>,
        ) {
            let tx = res.expect("tx open failed");
            let mut stores: Vec<String> = tx.object_store_names().collect();
            stores.sort();

            assert_eq!(tx.mode(), mode, "Mode");
            assert_eq!(stores, exp, "Stores");
        }

        async fn open_db() -> IdbDatabase {
            let mut req = IdbDatabase::open(&db_name()).expect("open");
            req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
                evt.db().create_object_store("s1")?;
                evt.db().create_object_store("s2")?;
                Ok(())
            }));
            req.await.expect("db await 1")
        }

        test_case!(async transaction_on_one => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one("s1"),
                IdbTransactionMode::Readonly,
                vec![String::from("s1")]
            );
        });

        test_case!(async transaction_on_multi_with_one => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_multi(&["s1"]),
                IdbTransactionMode::Readonly,
                vec![String::from("s1")]
            );
        });

        test_case!(async transaction_on_multi_with_multi => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_multi(&["s1", "s2"]),
                IdbTransactionMode::Readonly,
                vec![String::from("s1"), String::from("s2")]
            );
        });

        test_case!(async transaction_on_one_with_mode_r => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one_with_mode("s2", IdbTransactionMode::Readonly),
                IdbTransactionMode::Readonly,
                vec![String::from("s2")]
            );
        });

        test_case!(async transaction_on_one_with_mode_rw => {
            let db = open_db().await;
            check_transaction(
                db.transaction_on_one_with_mode("s2", IdbTransactionMode::Readwrite),
                IdbTransactionMode::Readwrite,
                vec![String::from("s2")]
            );
        });
    }

    test_case!(async create_object_store_with_params => {
        let mut req = IdbDatabase::open(&db_name()).expect("req");
        req.set_on_upgrade_needed(Some(move |evt: &IdbVersionChangeEvent| {
            evt.db().create_object_store_with_params(
                "s1",
                IdbObjectStoreParameters::new()
                .auto_increment(true)
                .key_path(Some(&IdbKeyPath::str("foo")))
            )?;
            Ok(())
        }));
        let db = req.await.expect("db");
        let tx = db.transaction_on_one("s1").expect("tx");
        let store = tx.object_store("s1").expect("store");

        assert_eq!(store.key_path(), Some(IdbKeyPath::str("foo")), "key path");
        assert!(store.auto_increment(), "auto_icrement");
    });
}
