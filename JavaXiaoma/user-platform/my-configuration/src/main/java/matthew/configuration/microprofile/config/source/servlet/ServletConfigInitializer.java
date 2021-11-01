package matthew.configuration.microprofile.config.source.servlet;

import javax.servlet.ServletContainerInitializer;
import javax.servlet.ServletContext;
import javax.servlet.ServletException;
import java.util.Set;

/**
 * @author Matthew
 * @time 2021/11/1 13:07
 */
public class ServletConfigInitializer implements ServletContainerInitializer {

    @Override
    public void onStartup(Set<Class<?>> c, ServletContext ctx) throws ServletException {
        // 增加 ServletContextListener
        ctx.addListener(ServletContextConfigInitializer.class);
    }
}
