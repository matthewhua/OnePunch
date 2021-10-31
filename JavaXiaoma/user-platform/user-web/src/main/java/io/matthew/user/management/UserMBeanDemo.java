package io.matthew.user.management;

import io.matthew.user.domain.User;

import javax.management.MBeanServer;
import javax.management.ObjectName;
import java.lang.management.ManagementFactory;

/**
 * @author Matthew
 * @date 2021-10-31 23:18
 */
public class UserMBeanDemo
{
	public static void main(String[] args) throws Exception
	{
		// 获取平台 MBean Server
		final MBeanServer mBeanServer = ManagementFactory.getPlatformMBeanServer();
		// 为 UserMXBean 定义 ObjectName
		final ObjectName objectName = new ObjectName("io/matthew/user/domain/management:type=User");
		// 创建UserMBean 实例
		final User user = new User();
		mBeanServer.registerMBean(createUserMBean(user), objectName);
		while (true) {
			Thread.sleep(2000);
			System.out.println(user);
		}
	}

	private static Object createUserMBean(User user) throws Exception {
		return new UserManager(user);
	}
}
