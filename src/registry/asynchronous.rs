use std::{
    iter::Rev,
    ops::Range,
    pin::Pin,
    task::{Context, Poll},
};

use super::{L10nRegistry, L10nRegistryLocked};
use crate::{
    fluent::FluentBundle,
    source::{RcResourceOption, ResourceStatus},
};

use futures::{
    ready,
    stream::{Collect, FuturesOrdered},
    FutureExt, Stream, StreamExt,
};
use unic_langid::LanguageIdentifier;

pub type ResourceSetStream = Collect<FuturesOrdered<ResourceStatus>, Vec<RcResourceOption>>;

impl<'a> L10nRegistryLocked<'a> {
    pub(crate) fn generate_resource_set<P>(
        &self,
        langid: &LanguageIdentifier,
        source_order: &[usize],
        resource_ids: &[P],
    ) -> ResourceSetStream
    where
        P: AsRef<str>,
    {
        debug_assert_eq!(source_order.len(), resource_ids.len());
        let stream = source_order
            .iter()
            .zip(resource_ids.iter().map(AsRef::as_ref))
            .map(|(&idx, path)| self.source_idx(idx).fetch_file(langid, path))
            .collect::<FuturesOrdered<_>>();
        stream.collect()
    }
}

impl L10nRegistry {
    pub fn generate_bundles_for_lang(
        &self,
        langid: LanguageIdentifier,
        resource_ids: Vec<String>,
    ) -> GenerateBundles {
        let lang_ids = vec![langid];

        GenerateBundles::new(self.clone(), lang_ids, resource_ids)
    }

    pub fn generate_bundles(
        &self,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> GenerateBundles {
        GenerateBundles::new(self.clone(), lang_ids, resource_ids)
    }
}

struct State {
    lang_id: LanguageIdentifier,
    source_orders: itertools::MultiProduct<Rev<Range<usize>>>,
    resource_set: Option<ResourceSetStream>,
}

pub struct GenerateBundles {
    reg: L10nRegistry,
    lang_ids: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    resource_ids: Vec<String>,
    state: Option<State>,
}

impl GenerateBundles {
    fn new(
        reg: L10nRegistry,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> Self {
        Self {
            reg,
            lang_ids: lang_ids.into_iter(),
            resource_ids,
            state: None,
        }
    }
}

impl Stream for GenerateBundles {
    type Item = FluentBundle;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        // ZOMG, this is torturous...
        // Effectively this is tracking state for performing the following loop:
        // ```
        // for langid in self.lang_ids {
        //     let source_orders = permutation of # of sources;
        //     for source_order in source_orders {
        //         let set = registry.generate_resource_set(langid, source_order, self.resource_ids).await;
        //         if Some(set) = set {
        //             return Some(Bundle::new(set))
        //         }
        //     }
        // }
        // ```
        loop {
            // Do we have state from last time?
            if let Some(State {
                lang_id,
                source_orders,
                resource_set,
            }) = &mut this.state
            {
                'inner: loop {
                    // Loop over all the source order combinations...
                    if let Some(fut) = resource_set {
                        // We have a pending Future to produce <Vec<Option<FluentResource>>>. Poll it...
                        let set = ready!(fut.poll_unpin(cx));
                        let _ = resource_set.take(); // A result is ready, clear the future.
                                                     // Construct Bundle from the Resources in the set.
                        let mut bundle = FluentBundle::new(vec![lang_id.clone()]);
                        for res in set {
                            if let Some(res) = res {
                                // TODO: add_resource returns `Result`
                                // this could become a `TryStream`
                                bundle
                                    .add_resource(res.res)
                                    .expect("Failed to add resource to bundle");
                            } else {
                                continue 'inner;
                            }
                        }
                        return Some(bundle).into();
                    }

                    // No pending Future, create the next one...
                    if let Some(source_order) = source_orders.next() {
                        resource_set.replace(this.reg.lock().generate_resource_set(
                            lang_id,
                            &source_order,
                            &this.resource_ids,
                        ));
                    } else {
                        break 'inner;
                    }
                }
            }

            // Move to the next LanguageIdentifier and reset the source permutation...
            if let Some(lang_id) = this.lang_ids.next() {
                let source_orders =
                    super::permute_iter(this.reg.lock().len(), this.resource_ids.len());
                this.state = Some(State {
                    lang_id,
                    source_orders,
                    resource_set: None,
                })
            } else {
                // No lang_id remaining. All done!
                return None.into();
            }
        }
    }
}

pub struct GenerateVec {
    reg: L10nRegistry,
    lang_ids: <Vec<LanguageIdentifier> as IntoIterator>::IntoIter,
    resource_ids: Vec<String>,
    state: Option<State>,
}

impl GenerateVec {
    pub fn new(
        reg: L10nRegistry,
        lang_ids: Vec<LanguageIdentifier>,
        resource_ids: Vec<String>,
    ) -> Self {
        Self {
            reg,
            lang_ids: lang_ids.into_iter(),
            resource_ids,
            state: None,
        }
    }
}

impl Stream for GenerateVec {
    type Item = Vec<RcResourceOption>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        // ZOMG, this is torturous...
        // Effectively this is tracking state for performing the following loop:
        // ```
        // for langid in self.lang_ids {
        //     let source_orders = permutation of # of sources;
        //     for source_order in source_orders {
        //         let set = registry.generate_resource_set(langid, source_order, self.resource_ids).await;
        //         if Some(set) = set {
        //             return Some(Bundle::new(set))
        //         }
        //     }
        // }
        // ```
        loop {
            // Do we have state from last time?
            if let Some(State {
                lang_id,
                source_orders,
                resource_set,
            }) = &mut this.state
            {
                'inner: loop {
                    // Loop over all the source order combinations...
                    if let Some(fut) = resource_set {
                        // We have a pending Future to produce Option<Vec<FluentResource>>. Poll it...
                        let set = ready!(fut.poll_unpin(cx));
                        let _ = resource_set.take(); // A result is ready, clear the future.
                        return Some(set).into();
                    }

                    // No pending Future, create the next one...
                    if let Some(source_order) = source_orders.next() {
                        resource_set.replace(this.reg.lock().generate_resource_set(
                            lang_id,
                            &source_order,
                            &this.resource_ids,
                        ));
                    } else {
                        break 'inner;
                    }
                }
            }

            // Move to the next LanguageIdentifier and reset the source permutation...
            if let Some(lang_id) = this.lang_ids.next() {
                let source_orders =
                    super::permute_iter(this.reg.lock().len(), this.resource_ids.len());
                this.state = Some(State {
                    lang_id,
                    source_orders,
                    resource_set: None,
                })
            } else {
                // No lang_id remaining. All done!
                return None.into();
            }
        }
    }
}
