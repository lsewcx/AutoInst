#include <iostream>
#include <string>
#include <vector>
#include <cstdlib>
#include <unordered_map>
#include <unordered_set>
#include <functional>

/**
 * 判断工具是否在支持列表中。
 *
 * 参数：
 *   supportedTools: 支持自动安装的工具集合。
 *   toolName: 需要检查的工具名称。
 *
 * 返回：
 *   工具受支持时返回 true，否则返回 false。
 */
bool isToolSupported(const std::unordered_set<std::string> &supportedTools, const std::string &toolName)
{
    return supportedTools.find(toolName) != supportedTools.end();
}

/**
 * 执行系统命令并处理失败信息。
 *
 * 参数：
 *   command: 需要执行的系统命令。
 *   errorMessage: 命令执行失败时显示的信息。
 *
 * 返回：
 *   命令执行成功时返回 true，否则返回 false。
 */
bool runCommand(const std::string &command, const std::string &errorMessage)
{
    int commandResult = std::system(command.c_str());

    if (commandResult != 0)
    {
        std::cout << errorMessage << std::endl;
        return false;
    }

    return true;
}

/**
 * 确保当前进程只执行一次 apt 软件源更新。
 *
 * 返回：
 *   更新成功或已经更新时返回 true，否则返回 false。
 */
bool ensureAptUpdated()
{
#if defined(__linux__)
    static bool hasUpdatedApt = false;

    if (hasUpdatedApt)
    {
        return true;
    }

    std::cout << "running sudo apt update..." << std::endl;

    if (!runCommand("sudo apt update", "failed to run sudo apt update"))
    {
        return false;
    }

    hasUpdatedApt = true;
    return true;
#else
    std::cout << "apt update only supports Linux apt environments" << std::endl;
    return false;
#endif
}

/**
 * 确保指定的 apt 软件包已经安装。
 *
 * 参数：
 *   packageName: 需要检查并安装的软件包或命令名称。
 *
 * 返回：
 *   软件包可用时返回 true，否则返回 false。
 */
bool ensurePackageInstalled(const std::string &packageName)
{
#if defined(__linux__)
    std::unordered_map<std::string, std::string> packageInstallNames;
    packageInstallNames["lsb_release"] = "lsb-release";

    std::string installPackageName = packageName;
    std::unordered_map<std::string, std::string>::iterator packageEntry = packageInstallNames.find(packageName);

    if (packageEntry != packageInstallNames.end())
    {
        installPackageName = packageEntry->second;
    }

    std::string checkCommand = "command -v " + packageName + " >/dev/null 2>&1";
    int checkResult = std::system(checkCommand.c_str());

    if (checkResult == 0)
    {
        std::cout << packageName << " is already installed" << std::endl;
        return true;
    }

    std::cout << packageName << " not found, installing " << packageName << "..." << std::endl;

    if (!ensureAptUpdated())
    {
        return false;
    }

    std::string installCommand = "sudo apt install -y " + installPackageName;
    int installResult = std::system(installCommand.c_str());

    if (installResult != 0)
    {
        std::cout << "failed to install " << packageName << std::endl;
        return false;
    }

    int verifyResult = std::system(checkCommand.c_str());

    if (verifyResult != 0)
    {
        std::cout << packageName << " install finished but command is still unavailable" << std::endl;
        return false;
    }

    std::cout << packageName << " installed successfully" << std::endl;
    return true;
#else
    std::cout << "auto install only supports Linux apt environments" << std::endl;
    return false;
#endif
}

/**
 * 安装指定工具所需的系统依赖。
 *
 * 参数：
 *   toolDependencies: 工具名称与依赖列表的映射。
 *   toolName: 需要安装依赖的工具名称。
 *
 * 返回：
 *   所有依赖均可用时返回 true，否则返回 false。
 */
bool ensureToolDependenciesInstalled(const std::unordered_map<std::string, std::vector<std::string>> &toolDependencies, const std::string &toolName)
{
    std::unordered_map<std::string, std::vector<std::string>>::const_iterator dependencyGroup = toolDependencies.find(toolName);

    if (dependencyGroup == toolDependencies.end())
    {
        return true;
    }

    for (const std::string &packageName : dependencyGroup->second)
    {
        if (!ensurePackageInstalled(packageName))
        {
            return false;
        }
    }

    return true;
}

/**
 * 下载并安装 ossutil。
 */
void installOssutil()
{
#if defined(__linux__)
    std::string packageFileName = "ossutil-2.2.2-linux-amd64.zip";
    std::string packageUrl = "https://gosspublic.alicdn.com/ossutil/v2/2.2.2/ossutil-2.2.2-linux-amd64.zip";
    std::string extractDirectory = "ossutil-2.2.2-linux-amd64";
    std::string binaryFilePath = extractDirectory + "/ossutil";
    std::string downloadCommand = "curl -L -o " + packageFileName + " " + packageUrl;
    std::string unzipCommand = "unzip -o " + packageFileName;
    std::string chmodCommand = "chmod 755 " + binaryFilePath;
    std::string moveCommand = "sudo mv " + binaryFilePath + " /usr/local/bin/ossutil";
    std::string linkCommand = "sudo ln -sf /usr/local/bin/ossutil /usr/bin/ossutil";
    std::string verifyCommand = "command -v ossutil >/dev/null 2>&1";

    std::cout << "Installing ossutil..." << std::endl;

    if (!runCommand(downloadCommand, "failed to download ossutil"))
    {
        return;
    }

    if (!runCommand(unzipCommand, "failed to unzip ossutil package"))
    {
        return;
    }

    if (!runCommand(chmodCommand, "failed to update ossutil permissions"))
    {
        return;
    }

    if (!runCommand(moveCommand, "failed to move ossutil into /usr/local/bin"))
    {
        return;
    }

    if (!runCommand(linkCommand, "failed to create ossutil symlink"))
    {
        return;
    }

    if (!runCommand(verifyCommand, "ossutil install finished but command is still unavailable"))
    {
        return;
    }

    std::cout << "ossutil installed successfully" << std::endl;
#else
    std::cout << "ossutil install only supports Linux apt environments" << std::endl;
#endif
}

/**
 * 下载并安装 Miniconda。
 */
void installMiniconda()
{
#if defined(__linux__)
    std::string installerFilePath = "/tmp/miniconda.sh";
    std::string installerUrl = "https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh";
    std::string installPath = "$HOME/miniconda3";
    std::string downloadCommand = "wget " + installerUrl + " -O " + installerFilePath;
    std::string installCommand = "bash " + installerFilePath + " -b -p " + installPath;
    std::string initCommand = installPath + "/bin/conda init bash";
    std::string verifyCommand = installPath + "/bin/conda --version >/dev/null 2>&1";

    std::cout << "Installing Miniconda..." << std::endl;

    if (!runCommand(downloadCommand, "failed to download Miniconda installer"))
    {
        return;
    }

    if (!runCommand(installCommand, "failed to install Miniconda"))
    {
        return;
    }

    if (!runCommand(initCommand, "failed to run conda init bash"))
    {
        return;
    }

    if (!runCommand(verifyCommand, "Miniconda install finished but conda is still unavailable"))
    {
        return;
    }

    std::cout << "Miniconda installed successfully." << std::endl;
    std::cout << "open a new shell or run: source ~/.bashrc" << std::endl;
#else
    std::cout << "Miniconda install only supports Linux environments" << std::endl;
#endif
}

/**
 * 配置 Docker 软件源并安装 Docker。
 */
void installDocker()
{
#if defined(__linux__)
    const char *userNameValue = std::getenv("SUDO_USER");

    if (userNameValue == nullptr || std::string(userNameValue).empty())
    {
        userNameValue = std::getenv("USER");
    }

    std::string userName;

    if (userNameValue != nullptr)
    {
        userName = userNameValue;
    }

    std::string keyringPath = "/usr/share/keyrings/docker-archive-keyring.gpg";
    std::string repositoryFilePath = "/etc/apt/sources.list.d/docker.list";
    std::string keyringCommand = "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor --yes -o " + keyringPath;
    std::string repositoryCommand = "echo \"deb [arch=$(dpkg --print-architecture) signed-by=" + keyringPath + "] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable\" | sudo tee " + repositoryFilePath + " > /dev/null";
    std::string updateCommand = "sudo apt update";
    std::string installCommand = "sudo apt install -y docker-ce docker-ce-cli containerd.io";
    std::string verifyCommand = "docker --version >/dev/null 2>&1";

    std::cout << "Installing Docker..." << std::endl;

    if (!runCommand(keyringCommand, "failed to install Docker GPG key"))
    {
        return;
    }

    if (!runCommand(repositoryCommand, "failed to add Docker apt repository"))
    {
        return;
    }

    if (!runCommand(updateCommand, "failed to run sudo apt update for Docker repository"))
    {
        return;
    }

    if (!runCommand(installCommand, "failed to install Docker"))
    {
        return;
    }

    if (!userName.empty() && userName != "root")
    {
        std::string groupCommand = "sudo usermod -aG docker " + userName;

        if (!runCommand(groupCommand, "failed to add user to docker group"))
        {
            return;
        }
    }

    if (!runCommand(verifyCommand, "Docker install finished but docker is still unavailable"))
    {
        return;
    }

    std::cout << "Docker installed successfully." << std::endl;

    if (!userName.empty() && userName != "root")
    {
        std::cout << "open a new shell or run: newgrp docker" << std::endl;
    }
#else
    std::cout << "Docker install only supports Linux apt environments" << std::endl;
#endif
}

/**
 * 下载并安装 tosutil。
 */
void installTosutil()
{
#if defined(__linux__)
    std::string fileName = "tosutil";
    std::string downloadUrl = "https://m645b3e1bb36e-mrap.mrap.accesspoint.tos-global.volces.com/linux/amd64/tosutil";
    std::string downloadCommand = "wget " + downloadUrl + " -O " + fileName;
    std::string chmodCommand = "chmod a+x " + fileName;
    std::string moveCommand = "sudo mv " + fileName + " /usr/local/bin/tosutil";
    std::string linkCommand = "sudo ln -sf /usr/local/bin/tosutil /usr/bin/tosutil";
    std::string verifyCommand = "command -v tosutil >/dev/null 2>&1";

    std::cout << "Installing tosutil..." << std::endl;

    if (!runCommand(downloadCommand, "failed to download tosutil"))
    {
        return;
    }

    if (!runCommand(chmodCommand, "failed to update tosutil permissions"))
    {
        return;
    }

    if (!runCommand(moveCommand, "failed to move tosutil into /usr/local/bin"))
    {
        return;
    }

    if (!runCommand(linkCommand, "failed to create tosutil symlink"))
    {
        return;
    }

    if (!runCommand(verifyCommand, "tosutil install finished but command is unavailable"))
    {
        return;
    }

    std::cout << "tosutil installed successfully." << std::endl;
#else
    std::cout << "tosutil install only supports Linux environments" << std::endl;
#endif
}

/**
 * 打印命令行帮助信息。
 */
void printHelp()
{
    std::cout << "Usage:" << std::endl;
    std::cout << "  autoinstall install <tool|all>" << std::endl;
    std::cout << "  autoinstall --help" << std::endl;
    std::cout << std::endl;
    std::cout << "Supported tools:" << std::endl;
    std::cout << "  ossutil" << std::endl;
    std::cout << "  miniconda" << std::endl;
    std::cout << "  docker" << std::endl;
    std::cout << "  tosutil" << std::endl;
    std::cout << "  all (install all supported tools)" << std::endl;
    std::cout << std::endl;
    std::cout << "Examples:" << std::endl;
    std::cout << "  autoinstall install docker" << std::endl;
    std::cout << "  autoinstall install all" << std::endl;
}

/**
 * 安装指定工具及其依赖。
 *
 * 参数：
 *   supportedTools: 支持自动安装的工具集合。
 *   toolName: 需要安装的工具名称。
 */
void installTool(const std::unordered_set<std::string> &supportedTools, const std::string &toolName)
{
    if (toolName.empty())
    {
        std::cout << "tool name is required" << std::endl;
        printHelp();
        return;
    }

    if (!isToolSupported(supportedTools, toolName))
    {
        std::cout << "unsupported tool: " << toolName << std::endl;
        printHelp();
        return;
    }

    std::unordered_map<std::string, std::vector<std::string>> toolDependencies;
    toolDependencies["ossutil"] = {"curl", "unzip"};
    toolDependencies["miniconda"] = {"wget"};
    toolDependencies["docker"] = {"curl", "gpg", "lsb_release"};
    toolDependencies["tosutil"] = {"wget"};

    if (!ensureToolDependenciesInstalled(toolDependencies, toolName))
    {
        return;
    }

    std::unordered_map<std::string, std::function<void()>> toolInstallers;

    toolInstallers["ossutil"] = []()
    {
        installOssutil();
    };

    toolInstallers["miniconda"] = []()
    {
        installMiniconda();
    };

    toolInstallers["docker"] = []()
    {
        installDocker();
    };

    toolInstallers["tosutil"] = []()
    {
        installTosutil();
    };

    std::unordered_map<std::string, std::function<void()>>::iterator installer = toolInstallers.find(toolName);

    if (installer == toolInstallers.end())
    {
        std::cout << "installer not implemented: " << toolName << std::endl;
        return;
    }

    installer->second();
}

/**
 * 按固定顺序安装所有受支持的工具。
 *
 * 参数：
 *   supportedTools: 支持自动安装的工具集合。
 */
void installAllTools(const std::unordered_set<std::string> &supportedTools)
{
    std::vector<std::string> toolNames = {"ossutil", "miniconda", "docker", "tosutil"};

    for (const std::string &toolName : toolNames)
    {
        std::cout << std::endl;
        std::cout << "installing " << toolName << "..." << std::endl;
        installTool(supportedTools, toolName);
    }
}

/**
 * 解析命令行参数并执行对应操作。
 *
 * 参数：
 *   argc: 命令行参数数量。
 *   argv: 命令行参数数组。
 *
 * 返回：
 *   命令执行成功时返回 0，命令格式错误时返回 1。
 */
int main(int argc, char *argv[])
{
    std::string action;
    std::string toolName;
    std::unordered_set<std::string> supportedTools = {"ossutil", "miniconda", "docker", "tosutil"};

    if (argc > 1)
    {
        action = argv[1];
    }

    if (argc > 2)
    {
        toolName = argv[2];
    }

    if (action == "--help" && argc == 2)
    {
        printHelp();
        return 0;
    }

    if (action == "install" && argc != 3)
    {
        std::cout << "invalid install command" << std::endl;
        printHelp();
        return 1;
    }

    std::unordered_map<std::string, std::function<void()>> actionHandlers;

    actionHandlers["install"] = [&supportedTools, &toolName]()
    {
        if (toolName == "all")
        {
            installAllTools(supportedTools);
            return;
        }

        installTool(supportedTools, toolName);
    };

    std::unordered_map<std::string, std::function<void()>>::iterator handler = actionHandlers.find(action);

    if (handler == actionHandlers.end())
    {
        if (action.empty())
        {
            std::cout << "command is required" << std::endl;
        }
        else
        {
            std::cout << "unsupported action: " << action << std::endl;
        }

        printHelp();
        return 1;
    }

    handler->second();

    return 0;
}
