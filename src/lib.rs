/// 受支持工具的固定安装顺序。
pub const SUPPORTED_TOOLS: [&str; 8] = [
    "ossutil",
    "miniconda",
    "nvm",
    "node",
    "pm2",
    "ffmpeg",
    "docker",
    "tosutil",
];

/// 判断工具是否在支持列表中。
///
/// Args:
///     tool_name: 需要检查的工具名称。
///
/// Returns:
///     工具受支持时返回 `true`，否则返回 `false`。
pub fn is_tool_supported(tool_name: &str) -> bool {
    SUPPORTED_TOOLS.contains(&tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证支持列表能够识别已知和未知工具。
    #[test]
    fn recognizes_supported_tools() {
        for tool_name in SUPPORTED_TOOLS {
            assert!(is_tool_supported(tool_name));
        }
        assert!(!is_tool_supported("unknown"));
    }
}
