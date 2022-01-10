package io.matthew.poidemo.xls;

import org.apache.poi.hssf.usermodel.*;

import java.io.FileOutputStream;
import java.io.IOException;

/**
 * 示范建立 如何建立并设置字体大小
 */
public class WorkingWithFonts {

    public static void main(String[] args) throws IOException {
        try (HSSFWorkbook wb = new HSSFWorkbook()) {
            HSSFSheet sheet = wb.createSheet("new sheet");

            //Create a row and put some cells in it, Rows are 0 based.
            final HSSFRow row = sheet.createRow(1);
            // Create a new font and alter it.
            final HSSFFont font = wb.createFont();
            font.setFontHeightInPoints((short) 24);
            font.setFontName("Jetbrains Mono");
            font.setItalic(true);
            font.setStrikeout(true);

            // Fonts are set into a style so create a new one to use.
            final HSSFCellStyle style = wb.createCellStyle();
            style.setFont(font);
            // Create a cell and put a value in it.
            HSSFCell cell = row.createCell(1);
            cell.setCellValue("This is a test of fonts");
            cell.setCellStyle(style);

            // Write the output to a file
            try(FileOutputStream fileout = new FileOutputStream("workbook2.xls")){
                wb.write(fileout);
            }
        }
    }
}
