package matthew.configuration.microprofile.config.source.servlet;

import matthew.configuration.microprofile.config.source.MapBasedConfigSource;

import javax.servlet.ServletContext;
import java.util.Enumeration;
import java.util.Map;

import static java.lang.String.format;

/**
 * @author Matthew
 * @time 2021/11/1 13:24
 */
public class ServletContextConfigSource extends MapBasedConfigSource {

    private final ServletContext servletContext;


    public ServletContextConfigSource(ServletContext servletContext) {
        super(format("ServletContext[path:%s] Init Parameters", servletContext.getContextPath()), 500);
        this.servletContext = servletContext;
    }


    @Override
    protected void prepareConfigData(Map configData) throws Throwable {
        Enumeration<String> parameterNames = servletContext.getInitParameterNames();
        while (parameterNames.hasMoreElements()) {
            String parameterName = parameterNames.nextElement();
            configData.put(parameterName, servletContext.getInitParameter(parameterName));
        }
    }
}
