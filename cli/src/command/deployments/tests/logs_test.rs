use crate::command::deployments::logs::LogsArgs;
use crate::command::deployments::services::Service;
use clap::ValueEnum;

#[test]
fn test_logs_args_structure() {
    let args = LogsArgs {
        project: "test-project".to_string(),
        service: Service::from_str("katana", true).unwrap(),
        limit: 100,
        follow: false,
        since: None,
        region: None,
        replica: None,
        container: None,
    };

    assert_eq!(args.project, "test-project");
    assert_eq!(args.service.to_string(), "katana");
    assert_eq!(args.limit, 100);
    assert!(!args.follow);
    assert_eq!(args.region, None);
    assert_eq!(args.replica, None);
    assert_eq!(args.container, None);

    let args = LogsArgs {
        project: "test-project".to_string(),
        service: Service::from_str("katana", true).unwrap(),
        limit: 50,
        follow: true,
        since: None,
        region: Some("us-east4".to_string()),
        replica: Some("torii-0".to_string()),
        container: Some("litestream".to_string()),
    };

    assert_eq!(args.project, "test-project");
    assert_eq!(args.service.to_string(), "katana");
    assert_eq!(args.limit, 50);
    assert!(args.follow);
    assert_eq!(args.region, Some("us-east4".to_string()));
    assert_eq!(args.replica, Some("torii-0".to_string()));
    assert_eq!(args.container, Some("litestream".to_string()));
}
