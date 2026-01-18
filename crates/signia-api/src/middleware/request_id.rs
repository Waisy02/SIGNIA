use axum::http::Request;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

pub fn layer() -> tower::util::Either<
    tower::layer::util::Identity,
    tower::layer::util::Stack<PropagateRequestIdLayer, SetRequestIdLayer<MakeRequestUuid>>,
> {
    let set = SetRequestIdLayer::x_request_id(MakeRequestUuid);
    let propagate = PropagateRequestIdLayer::x_request_id();
    tower::util::Either::B(tower::layer::util::Stack::new(propagate, set))
}
