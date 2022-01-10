package io.matthew.poidemo.xlsx.streaming;


import org.apache.poi.ss.usermodel.Cell;
import org.apache.poi.ss.usermodel.CellStyle;
import org.apache.poi.ss.usermodel.HorizontalAlignment;
import org.apache.poi.ss.usermodel.Row;
import org.apache.poi.xssf.streaming.DeferredSXSSFSheet;
import org.apache.poi.xssf.streaming.DeferredSXSSFWorkbook;

import java.io.FileOutputStream;
import java.io.IOException;

/**
 31	 * This sample demonstrates how to use DeferredSXSSFWorkbook to generate workbooks in a streaming way.
 32	 * This approach reduces the use of temporary files and can be used to output to streams like
 33	 * HTTP response streams.
 34	 */
public class DeferredGeneration {

    public static void main(String[] args) throws IOException {
        try (DeferredSXSSFWorkbook wb = new DeferredSXSSFWorkbook()){
            DeferredSXSSFSheet sheet1 = wb.createSheet("new sheet");

            // cell styles should be created outside the row generator function
            CellStyle cellStyle = wb.createCellStyle();
            cellStyle.setAlignment(HorizontalAlignment.CENTER);

            sheet1.setRowGenerator((ssxSheet) -> {
                for (int i = 0; i < 10; i++) {
                    Row row = ssxSheet.createRow(i);
                    Cell cell = row.createCell(2); //调节 cell 的
                    cell.setCellStyle(cellStyle);
                    cell.setCellValue("value " + i);
                }
            });

            try (FileOutputStream files = new FileOutputStream("DeferredGeneration2.xlsx")) {
                wb.write(files);
                //writeAvoidingTempFiles was added as an experimental change in POI 5.1.0
                //wb.writeAvoidingTempFiles(fileOut);
            }finally {
                // the dispose call is necessary to ensure temp files are removed
                wb.dispose();
            }
            System.out.println("wrote DeferredGeneration2.xlsx");
        }
    }
}
