# Appender

简体中文 | [English](README.md)

## 介绍

`Appender`是用于添加、读取和导出附加资源的工具。

### `Appender`有什用？

- 最典型的就是一些软件可以把一些数据流文件生成 exe 文件，比如一些 mp3 生成器，Flash 生成器，以及用来做动画的
  S-demo。它们的作用就是将数据对 PE 进行捆绑;
- 可以用于安装包制作，使用本程序将安装包附加至自定义程序，安装时释放资源即可;
- 可以用于隐藏文件，比如将文件增加到图片等格式中;

### 增加的资源会占用运行内存吗？

- `Overlay`是在附加在文件后面的，不被映射到内存空间中的数据，它提供它自己的程序打开自己来读取
- `Overlay`只是数据它是不映射到内存的，它将被程序以打开自己的方式来读取数据

### 最大能增加多少资源？

- 4GB是所有便携式可执行程序(32位和64位PE)的硬限制
- 其他格式（如图片格式）一般无此限制

### 如何保证资源完整？

`Appender`在释放文件前会检查资源长度是否一致，在释放后也会进行二次检测。

## 使用

我们由`资源ID`来标记文件，`资源ID`可以为任意长度小于64的文本，注意不允许重复。

### 增加资源

`Appender.exe add <目标文件> <资源文件> <资源ID> [新文件]`

**参数说明**

| 参数       | 短参数 | 描述     |
|----------|-----|--------|
| `<目标文件>` | 无   | 目标文件路径 |
| `<资源文件>` | 无   | 资源文件路径 |
| `<资源ID>` | 无   | 资源ID   |
| `[新文件]`  | 无   | 新文件路径  |

**示例**:

- 基本使用: `Appender.exe add D:\Program.exe D:\file.zip Archive`
- 输出新文件: `Appender.exe add D:\Program.exe D:\file.zip Archive D:\Program-new.exe`
- 设置压缩(0-9等级): `Appender.exe add D:\Program.exe D:\file.zip Archive -c 5`

### 释放资源

`Appender.exe export <目标文件> <资源ID> <输出路径>`

**参数说明**

| 参数       | 短参数 | 描述     |
|----------|-----|--------|
| `<目标文件>` | 无   | 目标文件路径 |
| `<资源ID>` | 无   | 资源ID   |
| `<输出路径>` | 无   | 输出路径   |

**示例**:

- 指定输出路径(保留原文件名): `Appender.exe export D:\Program.exe Archive D:\`
- 指定输出路径(自定义文件名): `Appender.exe export D:\Program.exe Archive D:\file.zip`
- 输出到目标文件目录下: `Appender.exe export D:\Program.exe Archive file.zip`

### 查看资源

`Appender.exe list <目标文件>`

**参数说明**

| 参数     | 短参数  | 描述     |
|--------|------|--------|
| `目标文件` | 无    | 目标文件路径 |
| `--id` | `-i` | 资源ID   |

**示例**:

- 查看所有资源: `Appender.exe list D:\Program.exe`
- 查看指定资源: `Appender.exe list D:\Program.exe --id Archive`

### 删除资源

`Appender.exe remove <目标文件> <资源ID>`

**参数说明**

| 参数       | 短参数 | 描述     |
|----------|-----|--------|
| `<目标文件>` | 无   | 目标文件路径 |
| `<资源ID>` | 无   | 资源ID   |

**示例**:

- 删除资源: `Appender.exe remove D:\Program.exe Archive`

## 许可证 📝

[Apache License 2.0](LICENSE)
