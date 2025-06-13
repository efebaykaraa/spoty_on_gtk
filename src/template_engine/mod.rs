use std::collections::HashMap;
use std::fs;
use std::io;

pub fn render_template(template_name: &str, variables: HashMap<String, String>) -> Result<String, io::Error> {
    let template_path = format!("templates/{}", template_name);
    let template_content = fs::read_to_string(template_path)?;

    let rendered_content = replace_variables(template_content, variables);
    Ok(rendered_content)
}

fn replace_variables(content: String, variables: HashMap<String, String>) -> String {
    let mut rendered_content = content;
    for (key, value) in variables {
        let placeholder = format!("{{{{ {} }}}}", key);
        rendered_content = rendered_content.replace(&placeholder, &value);
    }
    rendered_content
}
