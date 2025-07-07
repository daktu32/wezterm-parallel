// WezTerm Multi-Process Development Framework - Template IPC Integration Test

use wezterm_parallel::{Message, TemplateInfo};
use serde_json;

#[tokio::test]
async fn test_template_ipc_messages() {
    // Test TemplateList message serialization
    let template_list_msg = Message::TemplateList;
    let json = serde_json::to_string(&template_list_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(template_list_msg, deserialized);

    // Test TemplateListResponse message
    let template_info = TemplateInfo {
        name: "test_template".to_string(),
        description: "Test template description".to_string(),
        author: "Test Author".to_string(),
        version: "1.0".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        layout_type: "Single".to_string(),
        pane_count: 1,
        auto_start_processes: true,
    };

    let template_response = Message::TemplateListResponse {
        templates: vec![template_info.clone()],
    };

    let json = serde_json::to_string(&template_response).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    
    if let Message::TemplateListResponse { templates } = deserialized {
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "test_template");
        assert_eq!(templates[0].pane_count, 1);
    } else {
        panic!("Expected TemplateListResponse message");
    }
}

#[tokio::test]
async fn test_template_get_messages() {
    // Test TemplateGet message
    let template_get = Message::TemplateGet {
        name: "basic".to_string(),
    };
    
    let json = serde_json::to_string(&template_get).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(template_get, deserialized);

    // Test TemplateGetResponse message
    let template_response = Message::TemplateGetResponse {
        template: Some("{\"name\":\"basic\"}".to_string()),
    };
    
    let json = serde_json::to_string(&template_response).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    
    if let Message::TemplateGetResponse { template } = deserialized {
        assert!(template.is_some());
        assert!(template.unwrap().contains("basic"));
    } else {
        panic!("Expected TemplateGetResponse message");
    }
}

#[tokio::test]
async fn test_template_create_delete_messages() {
    // Test TemplateCreate message
    let template_create = Message::TemplateCreate {
        name: "custom_template".to_string(),
        content: "{\"name\":\"custom\"}".to_string(),
    };
    
    let json = serde_json::to_string(&template_create).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();

    // Test TemplateCreateResponse message
    let create_response = Message::TemplateCreateResponse {
        success: true,
        error: None,
    };
    
    let json = serde_json::to_string(&create_response).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();

    // Test TemplateDelete message
    let template_delete = Message::TemplateDelete {
        name: "custom_template".to_string(),
    };
    
    let json = serde_json::to_string(&template_delete).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();

    // Test TemplateDeleteResponse message
    let delete_response = Message::TemplateDeleteResponse {
        success: false,
        error: Some("Template not found".to_string()),
    };
    
    let json = serde_json::to_string(&delete_response).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_template_info_structure() {
    let template_info = TemplateInfo {
        name: "web_dev".to_string(),
        description: "Web development template".to_string(),
        author: "System".to_string(),
        version: "1.0".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        layout_type: "FourPaneGrid".to_string(),
        pane_count: 4,
        auto_start_processes: true,
    };

    // Test serialization
    let json = serde_json::to_string(&template_info).unwrap();
    let deserialized: TemplateInfo = serde_json::from_str(&json).unwrap();
    
    assert_eq!(template_info.name, deserialized.name);
    assert_eq!(template_info.pane_count, deserialized.pane_count);
    assert_eq!(template_info.auto_start_processes, deserialized.auto_start_processes);
}