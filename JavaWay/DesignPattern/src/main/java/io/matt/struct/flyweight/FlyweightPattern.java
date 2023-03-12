package io.matt.struct.flyweight;


import java.util.ArrayList;
import java.util.List;

interface Device {
    void print(int portNum);
}


abstract class NetDevice implements Device {

    abstract String getName();

    @Override
    public void print(int portNum) {
        System.out.printf("NetDevice :%s  port %d\n", getName(), portNum);
    }
}

class Hub extends NetDevice {
    @Override
    String getName() {
        return "集线器";
    }
}

class Switch extends NetDevice {
    @Override
    String getName() {
        return "交换机";
    }
}

class NetDeviceFactory {
    private static final class InstanceHolder {
        private static final NetDeviceFactory instance = new NetDeviceFactory();
    }

    public static NetDeviceFactory getInstance() {
        return NetDeviceFactory.InstanceHolder.instance;
    }

    private List<Device> devicePool = new ArrayList<>(3);

    public NetDeviceFactory() {
        Hub hub = new Hub();
        Switch aSwitch = new Switch();
        devicePool.add(hub);
        devicePool.add(aSwitch);
    }

    public Device getDevice(char ch) {
        if (ch == 'S') {
            return devicePool.get(1);
        } else if (ch == 'H') {
            return devicePool.get(0);
        } else {
            System.out.println("wrong input!");
        }
        return null;
    }


}

public class FlyweightPattern {

    public static void main(String[] args) {
        NetDeviceFactory factory = NetDeviceFactory.getInstance();
        Device device1, device2, device3, device4;

        // 客户端1获取一个HUB
        device1 = factory.getDevice('H');
        device1.print(1);

        // 客户端2获取一个hub
        device2 = factory.getDevice('H');
        device2.print(2);

        // 判断两个hub是否是同一个
        System.out.println("判断两个hub是否是同一个:");
        System.out.printf("device1:%s \ndevice2:%s\n", device1, device2); // 地址是一个
        System.out.print("\n\n\n\n");

        device3 = factory.getDevice('S');
        device3.print(1);
        device4= factory.getDevice('S');
        device4.print(3);
        // 判断两个switch是否是同一个
        System.out.println("判断两个switch是否是同一个:");
        System.out.printf("device1:%s \ndevice2:%s\n", device3, device4); // 地址是一个
    }
}
