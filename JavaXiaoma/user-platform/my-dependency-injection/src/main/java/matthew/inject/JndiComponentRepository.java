package matthew.inject;

import matthew.function.ThrowableAction;

import javax.naming.Context;
import javax.naming.InitialContext;
import javax.naming.NamingException;
import java.util.Set;

/**
 * 基于 JNDI {@link ComponentRepository} 实现
 *
 * @author Matthew
 * @projectName Validation
 * @time 2021/10/20 10:22
 */
public class JndiComponentRepository extends AbstractComponentRepository{

    private static final String COMPONENT_ENV_CONTEXT_NAME = "java:comp/env";

    private Context envContext; // Component Env Context

    @Override
    protected Set<String> listComponentNames() {
        return null;
    }

    @Override
    protected Object doGetComponent(String s) {
        return null;
    }


    private void initEnvContext() throws RuntimeException{
        if (this.envContext != null)
            return;
        Context context = null;
        try {
            context = new InitialContext();
            this.envContext = (Context) context.lookup(COMPONENT_ENV_CONTEXT_NAME);
        } catch (NamingException e){
            throw new RuntimeException(e);
        }finally {
            close(context);
        }

    }

    private static void close(Context context) {
        if (context != null) {
            ThrowableAction.execute(context::close);
        }
    }
}

}
