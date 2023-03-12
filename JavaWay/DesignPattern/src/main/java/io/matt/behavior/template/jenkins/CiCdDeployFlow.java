package io.matt.behavior.template.jenkins;

public class CiCdDeployFlow extends DeployFlow{

    @Override
    public void pullCodeFromGitlab() {
        System.out.println("持续集成服务器将代码拉取到节点服务器上......");
    }

    @Override
    public void compileAndPackage() {
        System.out.println("自动进行编译&打包......");
    }

    @Override
    public void copyToTestServer() {
        System.out.println("自动将包拷贝到测试环境服务器......");
    }

    @Override
    public void testing() {
        System.out.println("执行自动化测试......");
    }

}
