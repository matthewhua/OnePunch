### 这是学习 文件服务器

文件的上传、下载功能是软件系统常见的功能，包括上传文件、下载文件、查看文件等。例如：电商系统中需要上传商品的图片、广告视频，办公系统中上传附件，社交类系统中上传用户头像

基于以上原因，微服务体系下的应用系统一般都有一个文件服务，用于统一管理文件上传下载等功能，大型电商系统甚至有独立的文件、图片、视频服务。此时架构体系变为：

![1585712078385](G:/BaiduNetdiskDownload/005-黑马精英进阶/阶段一 中台战略与组件化开发专题课程/中台组件课程配套资料/文件服务/讲义/img/1585712078385.png)

这种方式提供一个独立的文件微服务，该微服务向应用系统提供统一的上传、下载、查看接口，应用系统调用方式相同，并且屏蔽了底层对外调用OSS服务的接口，即使以后迁移OSS服务商，应用层面的系统也不需要变动。

这种模式也有一个小问题，比如我们调用了阿里云的OSS服务，如果所有的下载、查看功能都调用文件服务，那么文件服务的网络流量将会有非常大的压力。所以常用的做法是这样的：

![1585712407005](G:/BaiduNetdiskDownload/005-黑马精英进阶/阶段一 中台战略与组件化开发专题课程/中台组件课程配套资料/文件服务/讲义/img/1585712407005.png)