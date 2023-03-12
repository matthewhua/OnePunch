package io.matt.behavior.template.jenkins;

public abstract class DeployFlow {
    //使用final关键字来约束步骤不能轻易修改
    public final void buildFlow() {
        pullCodeFromGitlab(); //从GitLab上拉取代码
        compileAndPackage(); //编译打包
        copyToRemoteServer(); //部署测试环境
        testing();              //测试
        copyToRemoteServer();   //上传包到线上环境
        startApp();              //启动程序
    }

    public abstract void pullCodeFromGitlab();

    public abstract void compileAndPackage();

    public abstract void copyToTestServer();

    public abstract void testing();

    private void copyToRemoteServer() {
        System.out.println("统一自动上传 启动App包到对应线上服务器");
    }

    private void startApp() {
        System.out.println("统一自动 启动线上App");
    }
}
