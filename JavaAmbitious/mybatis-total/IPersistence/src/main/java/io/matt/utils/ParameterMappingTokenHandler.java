package io.matt.utils;

import java.util.ArrayList;
import java.util.List;

public class ParameterMappingTokenHandler implements TokenHandler{
    private List<ParameterMapping> parameterMappings = new ArrayList<>();

    @Override
    public String handleToken(String content) {
        parameterMappings.add(buildParameterMappings(content));
        return "?";
    }

    private ParameterMapping buildParameterMappings(String content){
        return new ParameterMapping(content);
    }

    public List<ParameterMapping> getParameterMappings(){
        return parameterMappings;
    }

    public void setParameterMappings(List<ParameterMapping> parameterMappings) {
        this.parameterMappings = parameterMappings;
    }
}
