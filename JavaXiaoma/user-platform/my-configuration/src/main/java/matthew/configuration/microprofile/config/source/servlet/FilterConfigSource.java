package matthew.configuration.microprofile.config.source.servlet;

import matthew.configuration.microprofile.config.source.MapBasedConfigSource;
import org.eclipse.microprofile.config.spi.ConfigSource;

import javax.servlet.FilterConfig;
import java.util.Enumeration;
import java.util.Map;

import static java.lang.String.format;

/**
 * @author Matthew
 * @time 2021/11/1 11:49
 *
 *  The {@link ConfigSource} implementation based on {@link FilterConfig}
 */
public class FilterConfigSource extends MapBasedConfigSource {

    private final FilterConfig filterConfig;


    public FilterConfigSource(String name, int ordinal, FilterConfig filterConfig) {
        super(format("Filter[name:%s] Init Parameters", filterConfig.getFilterName()), 500);
        this.filterConfig = filterConfig;
    }

    @Override
    protected void prepareConfigData(Map configData) throws Throwable {
        final Enumeration<String> initParameterNames = filterConfig.getInitParameterNames();
        while (initParameterNames.hasMoreElements()){
            final String parameterName = initParameterNames.nextElement();
            configData.put(parameterName, filterConfig.getInitParameter(parameterName));
        }
    }
}
