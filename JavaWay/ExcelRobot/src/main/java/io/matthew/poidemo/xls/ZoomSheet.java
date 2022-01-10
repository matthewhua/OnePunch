package io.matthew.poidemo.xls;

import org.apache.poi.hssf.usermodel.HSSFSheet;
import org.apache.poi.hssf.usermodel.HSSFWorkbook;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;

/**
 *Sets the zoom magnication for a sheet
 */
public class ZoomSheet {
    public static void main(String[] args) throws IOException {
       try ( HSSFWorkbook wb = new HSSFWorkbook();){
           HSSFSheet sheet = wb.createSheet("new sheet");
           sheet.setZoom(75); // 75 percent magnification

           try(FileOutputStream outputStream = new FileOutputStream("workbook.xls")){
               wb.write(outputStream);
           }
       }
    }
}
