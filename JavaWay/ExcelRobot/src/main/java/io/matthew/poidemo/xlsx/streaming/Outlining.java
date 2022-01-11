package io.matthew.poidemo.xlsx.streaming;

import org.apache.poi.xssf.streaming.SXSSFSheet;
import org.apache.poi.xssf.streaming.SXSSFWorkbook;

import java.io.FileOutputStream;
import java.io.IOException;

/**
 * 会有下横线
 */
public class Outlining {

    public static void main(String[] args) throws IOException {
        Outlining o = new Outlining();
        o.collapseRow();
    }

    private void collapseRow() throws IOException {
        try(SXSSFWorkbook wb2 = new SXSSFWorkbook (100)){
            final SXSSFSheet sheet = wb2.createSheet("new sheet");

            int rowCount = 20;
            for (int i = 0; i < rowCount; i++) {
                sheet.createRow(i);
            }

            sheet.groupRow(4, 9);
            sheet.groupRow(11, 19);

            sheet.setRowGroupCollapsed(4, true);

            try (FileOutputStream fileOut = new FileOutputStream("outlining_collapsed.xlsx")){
                wb2.write(fileOut);
            }finally {
                //the dispose call is necessary to ensure temp files are removed
                wb2.dispose();
            }
        }

    }
}
