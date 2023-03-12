package io.matt.behavior.template.jenkins;

public class Client {

    public static void main(String[] args) {
        System.out.println("开始本地手动发布流程 =======");
        LocalDeployFlow localDeployFlow = new LocalDeployFlow();
        localDeployFlow.buildFlow();
        System.out.println("********************");
        System.out.println("开始 CICD 发布流程======");
        CiCdDeployFlow ciCdDeployFlow = new CiCdDeployFlow();
        ciCdDeployFlow.buildFlow();
    }
}
