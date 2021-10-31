package matthew.configuration.microprofile.config.servlet.listener;

import org.eclipse.microprofile.config.Config;
import org.eclipse.microprofile.config.ConfigProvider;
import org.eclipse.microprofile.config.spi.ConfigProviderResolver;

import javax.servlet.ServletContext;
import javax.servlet.ServletRequest;
import javax.servlet.ServletRequestEvent;
import javax.servlet.ServletRequestListener;

/**
 * @author Matthew
 * @date 2021-10-31 16:44
 */
public class ConfigServletRequestListener implements ServletRequestListener
{
	private static final ThreadLocal<Config> configThreadLocal = new ThreadLocal<>();



	@Override public void requestInitialized(ServletRequestEvent sre)
	{
		ServletRequest request = sre.getServletRequest();
		ServletContext servletContext = request.getServletContext();
		ClassLoader classLoader = servletContext.getClassLoader();
		ConfigProviderResolver configProviderResolver = ConfigProviderResolver.instance();
		Config config = configProviderResolver.getConfig(classLoader);
		configThreadLocal.set(config);
	}


	@Override public void requestDestroyed(ServletRequestEvent sre)
	{
		// 防止 OOM
		configThreadLocal.remove();
	}


}
