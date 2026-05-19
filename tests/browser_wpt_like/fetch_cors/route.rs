use tetherscript::browser_agent::{RouteAction, RouteFulfillment};

pub fn preflight() -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 204,
        headers: vec![
            (
                "access-control-allow-origin".into(),
                "https://app.test".into(),
            ),
            ("access-control-allow-methods".into(), "POST".into()),
            ("access-control-allow-headers".into(), "x-test".into()),
        ],
        body: String::new(),
    })
}

pub fn cors_text(body: &str) -> RouteAction {
    RouteAction::Fulfill(RouteFulfillment {
        status: 200,
        headers: vec![(
            "access-control-allow-origin".into(),
            "https://app.test".into(),
        )],
        body: body.into(),
    })
}
