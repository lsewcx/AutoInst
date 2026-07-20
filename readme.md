# AutoInst

## 工具作用

AutoInst 是一个面向 Linux 环境的命令行自动安装工具。它会自动检查并安装目标工具所需的系统依赖，然后完成目标工具的下载、安装和可用性验证，减少手动执行多条安装命令的步骤。

## 使用方法

当前支持自动安装以下工具：

- ossutil
- miniconda
- nvm
- docker
- tosutil

使用示例：

```bash
./autoinstall install docker
```
