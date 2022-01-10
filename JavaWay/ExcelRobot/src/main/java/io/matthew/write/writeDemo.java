package io.matthew.write;

import com.alibaba.excel.EasyExcel;
import io.matthew.util.TestFileUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Date;
import java.util.List;

public class writeDemo {


    /**
     * 最简单的写
     * <p>1. 创建excel对应的实体对象 参照{@link }
     * <p>2. 直接写即可
     */
    @Test
    public void simpleWrite() {
        // 写法1
        String fileName = TestFileUtil.getPath() + "simpleWrite" + System.currentTimeMillis() + ".xlsx";
        // 这里 需要指定写用哪个class去写，然后写到第一个sheet，名字为模板 然后文件流会自动关闭
        // 如果这里想使用03 则 传入excelType参数即可
        // 分页查询数据
        EasyExcel.write(fileName, DemoData.class)
                .sheet("模板")
                .doWrite(data());
    }



    private List<DemoData> data(){
        return getDemoData();
    }


    static List<DemoData> getDemoData() {
        List<DemoData> list = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            DemoData demoData = new DemoData();
            demoData.setString("matthew 的女朋友" + i);
            demoData.setDate(new Date());
            demoData.setDoubleData(0.56);
            list.add(demoData);
        }
        return list;
    }
}
