use auto_core::{SUPPORTED_TOOLS, is_tool_supported};
use std::env;
#[cfg(target_os = "linux")]
use std::process::Command;
use std::process::ExitCode;

#[derive(Default)]
struct Installer {
    #[cfg(target_os = "linux")]
    apt_updated: bool,
}

impl Installer {
    /// 确保当前进程只成功执行一次 apt 软件源更新。
    ///
    /// Returns:
    ///     更新成功或已经更新时返回 `true`，否则返回 `false`。
    #[cfg(target_os = "linux")]
    fn ensure_apt_updated(&mut self) -> bool {
        if self.apt_updated {
            return true;
        }

        println!("running sudo apt update...");
        if !run_shell_command("sudo apt update", "failed to run sudo apt update") {
            return false;
        }

        self.apt_updated = true;
        true
    }

    /// 确保指定的 apt 软件包已经安装。
    ///
    /// Args:
    ///     package_name: 需要检查并安装的软件包或命令名称。
    ///
    /// Returns:
    ///     软件包可用时返回 `true`，否则返回 `false`。
    fn ensure_package_installed(&mut self, package_name: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            let install_package_name = match package_name {
                "lsb_release" => "lsb-release",
                _ => package_name,
            };
            let check_command = format!("command -v {package_name} >/dev/null 2>&1");

            if command_succeeds(&check_command) {
                println!("{package_name} is already installed");
                return true;
            }

            println!("{package_name} not found, installing {package_name}...");
            if !self.ensure_apt_updated() {
                return false;
            }

            let install_command = format!("sudo apt install -y {install_package_name}");
            if !run_shell_command(
                &install_command,
                &format!("failed to install {package_name}"),
            ) {
                return false;
            }

            if !command_succeeds(&check_command) {
                println!("{package_name} install finished but command is still unavailable");
                return false;
            }

            println!("{package_name} installed successfully");
            true
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = package_name;
            println!("auto install only supports Linux apt environments");
            false
        }
    }

    /// 安装指定工具所需的系统依赖。
    ///
    /// Args:
    ///     tool_name: 需要安装依赖的工具名称。
    ///
    /// Returns:
    ///     所有依赖均可用时返回 `true`，否则返回 `false`。
    fn ensure_tool_dependencies_installed(&mut self, tool_name: &str) -> bool {
        for package_name in tool_dependencies(tool_name) {
            if !self.ensure_package_installed(package_name) {
                return false;
            }
        }

        true
    }

    /// 下载并安装 ossutil。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_ossutil(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            println!("Installing ossutil...");
            let succeeded = run_shell_steps(&[
                (
                    "curl -L -o ossutil-2.2.2-linux-amd64.zip https://gosspublic.alicdn.com/ossutil/v2/2.2.2/ossutil-2.2.2-linux-amd64.zip",
                    "failed to download ossutil",
                ),
                (
                    "unzip -o ossutil-2.2.2-linux-amd64.zip",
                    "failed to unzip ossutil package",
                ),
                (
                    "chmod 755 ossutil-2.2.2-linux-amd64/ossutil",
                    "failed to update ossutil permissions",
                ),
                (
                    "sudo mv ossutil-2.2.2-linux-amd64/ossutil /usr/local/bin/ossutil",
                    "failed to move ossutil into /usr/local/bin",
                ),
                (
                    "sudo ln -sf /usr/local/bin/ossutil /usr/bin/ossutil",
                    "failed to create ossutil symlink",
                ),
                (
                    "command -v ossutil >/dev/null 2>&1",
                    "ossutil install finished but command is still unavailable",
                ),
            ]);

            if succeeded {
                println!("ossutil installed successfully");
            }
            succeeded
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("ossutil install only supports Linux apt environments");
            false
        }
    }

    /// 下载并安装 Miniconda。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_miniconda(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            println!("Installing Miniconda...");
            let succeeded = run_shell_steps(&[
                (
                    "wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O /tmp/miniconda.sh",
                    "failed to download Miniconda installer",
                ),
                (
                    "bash /tmp/miniconda.sh -b -p $HOME/miniconda3",
                    "failed to install Miniconda",
                ),
                (
                    "$HOME/miniconda3/bin/conda init bash",
                    "failed to run conda init bash",
                ),
                (
                    "$HOME/miniconda3/bin/conda --version >/dev/null 2>&1",
                    "Miniconda install finished but conda is still unavailable",
                ),
            ]);

            if succeeded {
                println!("Miniconda installed successfully.");
                println!("open a new shell or run: source ~/.bashrc");
            }
            succeeded
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("Miniconda install only supports Linux environments");
            false
        }
    }

    /// 下载并安装 nvm。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_nvm(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            println!("Installing nvm...");
            let verify_command =
                format!("{} && command -v nvm >/dev/null 2>&1", nvm_load_command());

            if !run_shell_command(
                "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.6/install.sh | bash",
                "failed to install nvm",
            ) || !run_shell_command(
                &verify_command,
                "nvm install finished but nvm is unavailable",
            ) {
                return false;
            }

            println!("nvm installed successfully.");
            println!("open a new shell to start using nvm");
            true
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("nvm install only supports Linux environments");
            false
        }
    }

    /// 检查 nvm 并安装最新版本 Node.js。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_node(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            let load_nvm_command = nvm_load_command();
            let check_nvm_command = format!("{load_nvm_command} && command -v nvm >/dev/null 2>&1");
            let install_command = format!("{load_nvm_command} && nvm install node");
            let verify_command = format!(
                "{load_nvm_command} && nvm use node >/dev/null && node --version >/dev/null 2>&1"
            );

            println!("Installing Node.js...");
            if !command_succeeds(&check_nvm_command) {
                println!("nvm not found, installing nvm...");
                self.install_nvm();
                if !run_shell_command(&check_nvm_command, "nvm is unavailable after installation") {
                    return false;
                }
            }

            if !run_shell_command(&install_command, "failed to install Node.js with nvm")
                || !run_shell_command(
                    &verify_command,
                    "Node.js install finished but node is unavailable",
                )
            {
                return false;
            }

            println!("Node.js installed successfully.");
            true
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("Node.js install only supports Linux environments");
            false
        }
    }

    /// 检查 npm 并全局安装 PM2。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_pm2(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            let load_npm_command = format!(
                "if ! command -v npm >/dev/null 2>&1; then {}; fi",
                nvm_load_command()
            );
            let check_npm_command = format!("{load_npm_command} && command -v npm >/dev/null 2>&1");
            let install_command = format!("{load_npm_command} && npm install -g pm2");
            let verify_command = format!(
                "{load_npm_command} && command -v pm2 >/dev/null 2>&1 && pm2 --version >/dev/null 2>&1"
            );

            println!("Installing PM2...");
            if !command_succeeds(&check_npm_command) {
                println!("npm not found, installing Node.js...");
                if !self.ensure_package_installed("curl") {
                    return false;
                }
                self.install_node();
                if !run_shell_command(
                    &check_npm_command,
                    "npm is unavailable after Node.js installation",
                ) {
                    return false;
                }
            }

            if !run_shell_command(&install_command, "failed to install PM2 with npm")
                || !run_shell_command(
                    &verify_command,
                    "PM2 install finished but pm2 is unavailable",
                )
            {
                return false;
            }

            println!("PM2 installed successfully.");
            true
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("PM2 install only supports Linux environments");
            false
        }
    }

    /// 使用 apt 安装 FFmpeg。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_ffmpeg(&mut self) -> bool {
        self.ensure_package_installed("ffmpeg")
    }

    /// 配置 Docker 软件源并安装 Docker。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_docker(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            let user_name = env::var("SUDO_USER")
                .ok()
                .filter(|value| !value.is_empty())
                .or_else(|| env::var("USER").ok())
                .unwrap_or_default();

            println!("Installing Docker...");
            if !run_shell_steps(&[
                (
                    "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor --yes -o /usr/share/keyrings/docker-archive-keyring.gpg",
                    "failed to install Docker GPG key",
                ),
                (
                    "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null",
                    "failed to add Docker apt repository",
                ),
                (
                    "sudo apt update",
                    "failed to run sudo apt update for Docker repository",
                ),
                (
                    "sudo apt install -y docker-ce docker-ce-cli containerd.io",
                    "failed to install Docker",
                ),
            ]) {
                return false;
            }

            if !user_name.is_empty()
                && user_name != "root"
                && !run_program(
                    "sudo",
                    &["usermod", "-aG", "docker", &user_name],
                    "failed to add user to docker group",
                )
            {
                return false;
            }

            if !run_shell_command(
                "docker --version >/dev/null 2>&1",
                "Docker install finished but docker is still unavailable",
            ) {
                return false;
            }

            println!("Docker installed successfully.");
            if !user_name.is_empty() && user_name != "root" {
                println!("open a new shell or run: newgrp docker");
            }
            true
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("Docker install only supports Linux apt environments");
            false
        }
    }

    /// 下载并安装 tosutil。
    ///
    /// Returns:
    ///     安装并验证成功时返回 `true`，否则返回 `false`。
    fn install_tosutil(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            println!("Installing tosutil...");
            let succeeded = run_shell_steps(&[
                (
                    "wget https://m645b3e1bb36e-mrap.mrap.accesspoint.tos-global.volces.com/linux/amd64/tosutil -O tosutil",
                    "failed to download tosutil",
                ),
                ("chmod a+x tosutil", "failed to update tosutil permissions"),
                (
                    "sudo mv tosutil /usr/local/bin/tosutil",
                    "failed to move tosutil into /usr/local/bin",
                ),
                (
                    "sudo ln -sf /usr/local/bin/tosutil /usr/bin/tosutil",
                    "failed to create tosutil symlink",
                ),
                (
                    "command -v tosutil >/dev/null 2>&1",
                    "tosutil install finished but command is unavailable",
                ),
            ]);

            if succeeded {
                println!("tosutil installed successfully.");
            }
            succeeded
        }

        #[cfg(not(target_os = "linux"))]
        {
            println!("tosutil install only supports Linux environments");
            false
        }
    }

    /// 安装指定工具及其依赖。
    ///
    /// Args:
    ///     tool_name: 需要安装的工具名称。
    ///
    /// Returns:
    ///     安装成功时返回 `true`，否则返回 `false`。
    fn install_tool(&mut self, tool_name: &str) -> bool {
        if !is_tool_supported(tool_name) {
            println!("unsupported tool: {tool_name}");
            print_help();
            return false;
        }

        if !self.ensure_tool_dependencies_installed(tool_name) {
            return false;
        }

        match tool_name {
            "ossutil" => self.install_ossutil(),
            "miniconda" => self.install_miniconda(),
            "nvm" => self.install_nvm(),
            "node" => self.install_node(),
            "pm2" => self.install_pm2(),
            "ffmpeg" => self.install_ffmpeg(),
            "docker" => self.install_docker(),
            "tosutil" => self.install_tosutil(),
            _ => false,
        }
    }

    /// 按固定顺序安装所有受支持的工具。
    fn install_all_tools(&mut self) {
        for tool_name in SUPPORTED_TOOLS {
            println!("\ninstalling {tool_name}...");
            self.install_tool(tool_name);
        }
    }
}

/// 返回指定工具所需的系统依赖。
///
/// Args:
///     tool_name: 需要查询依赖的工具名称。
///
/// Returns:
///     工具对应的依赖名称切片；没有依赖时返回空切片。
fn tool_dependencies(tool_name: &str) -> &'static [&'static str] {
    match tool_name {
        "ossutil" => &["curl", "unzip"],
        "miniconda" | "tosutil" => &["wget"],
        "nvm" | "node" => &["curl"],
        "docker" => &["curl", "gpg", "lsb_release"],
        _ => &[],
    }
}

/// 执行程序并处理失败信息。
///
/// Args:
///     program: 需要执行的程序名称。
///     arguments: 传递给程序的参数。
///     error_message: 执行失败时显示的信息。
///
/// Returns:
///     程序成功退出时返回 `true`，否则返回 `false`。
#[cfg(target_os = "linux")]
fn run_program(program: &str, arguments: &[&str], error_message: &str) -> bool {
    match Command::new(program).args(arguments).status() {
        Ok(status) if status.success() => true,
        Ok(_) => {
            println!("{error_message}");
            false
        }
        Err(error) => {
            println!("{error_message}: {error}");
            false
        }
    }
}

/// 使用 Bash 执行命令并处理失败信息。
///
/// Args:
///     command: 需要执行的 Bash 命令。
///     error_message: 执行失败时显示的信息。
///
/// Returns:
///     命令成功退出时返回 `true`，否则返回 `false`。
#[cfg(target_os = "linux")]
fn run_shell_command(command: &str, error_message: &str) -> bool {
    run_program("bash", &["-c", command], error_message)
}

/// 按顺序执行一组 Bash 命令。
///
/// Args:
///     steps: 命令及其失败提示组成的切片。
///
/// Returns:
///     所有命令均成功时返回 `true`，否则返回 `false`。
#[cfg(target_os = "linux")]
fn run_shell_steps(steps: &[(&str, &str)]) -> bool {
    steps
        .iter()
        .all(|(command, error_message)| run_shell_command(command, error_message))
}

/// 静默执行 Bash 命令并检查是否成功。
///
/// Args:
///     command: 需要执行的 Bash 命令。
///
/// Returns:
///     命令成功退出时返回 `true`，否则返回 `false`。
#[cfg(target_os = "linux")]
fn command_succeeds(command: &str) -> bool {
    Command::new("bash")
        .args(["-c", command])
        .status()
        .is_ok_and(|status| status.success())
}

/// 构建在 Bash 中加载 nvm 的命令。
///
/// Returns:
///     用于设置 `NVM_DIR` 并加载 nvm 的命令。
#[cfg(target_os = "linux")]
fn nvm_load_command() -> &'static str {
    "if [ -z \"${NVM_DIR-}\" ]; then \
     if [ -z \"${XDG_CONFIG_HOME-}\" ]; then NVM_DIR=\"${HOME}/.nvm\"; \
     else NVM_DIR=\"${XDG_CONFIG_HOME}/nvm\"; fi; fi; \
     export NVM_DIR; \
     [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\""
}

/// 打印命令行帮助信息。
fn print_help() {
    println!(
        "Usage:
  autoinstall install <tool|all>
  autoinstall --help

Supported tools:
  ossutil
  miniconda
  nvm
  node
  pm2
  ffmpeg
  docker
  tosutil
  all (install all supported tools)

Examples:
  autoinstall install node
  autoinstall install pm2
  autoinstall install ffmpeg
  autoinstall install docker
  autoinstall install all"
    );
}

/// 解析命令行变量并执行对应操作。
///
/// Args:
///     arguments: 不包含程序名称的命令行变量。
///
/// Returns:
///     命令格式正确时返回成功退出码，否则返回失败退出码。
fn run(arguments: &[String]) -> ExitCode {
    if arguments == ["--help"] {
        print_help();
        return ExitCode::SUCCESS;
    }

    if arguments.first().is_some_and(|action| action == "install") {
        if arguments.len() != 2 {
            println!("invalid install command");
            print_help();
            return ExitCode::FAILURE;
        }

        let mut installer = Installer::default();
        if arguments[1] == "all" {
            installer.install_all_tools();
        } else {
            installer.install_tool(&arguments[1]);
        }
        return ExitCode::SUCCESS;
    }

    match arguments.first() {
        Some(action) => println!("unsupported action: {action}"),
        None => println!("command is required"),
    }
    print_help();
    ExitCode::FAILURE
}

/// 读取命令行变量并启动程序。
///
/// Returns:
///     程序执行结果对应的退出码。
fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    run(&arguments)
}
