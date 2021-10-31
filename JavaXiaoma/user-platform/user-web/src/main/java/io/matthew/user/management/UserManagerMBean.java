package io.matthew.user.management;

import io.matthew.user.domain.User;

/**
 * @author Matthew
 * @date 2021-10-31 19:40
 *
 * {@link User} MBean 接口描述
 */
public interface UserManagerMBean
{

	// MBeanAttributeInfo 列表
	Long getId();

	void setId(Long id);

	String getName();

	void setName(String name);

	String getPassword();

	void setPassword(String password);

	String getEmail();

	void setEmail(String email);

	String getPhoneNumber();

	void setPhoneNumber(String phoneNumber);

	// MBeanOperationInfo
	String toString();

	User getUser();
}

