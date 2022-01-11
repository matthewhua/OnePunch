package io.matthew.poidemo.xlsx.usermodel;

import org.apache.poi.ss.usermodel.CellStyle;
import org.apache.poi.ss.usermodel.HorizontalAlignment;
import org.apache.poi.ss.usermodel.VerticalAlignment;
import org.apache.poi.xssf.usermodel.*;
import org.openxmlformats.schemas.spreadsheetml.x2006.main.impl.CTRowImpl;

import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.util.ArrayList;

/**
 * 显示各种对齐选项的工作原理。 由罗马尼亚的 Cristian Petrula
 * 于 2010 年 5 月 26 日修改 添加了新方法 centerAcrossSelection
 * 以使用HorizontalAlignment.CENTER_SELECTION在一个选择上居中列内容
 * 要创建此方法示例仅针对 XSSF 进行更改，并且之前的 AligningCells.java 示例已移至SS 示例文件夹。
 */
public class AligningCells {

    public static void main(String[] args) throws IOException {
        try (XSSFWorkbook wb = new XSSFWorkbook()) {

            XSSFSheet sheet = wb.createSheet();
            XSSFRow row = sheet.createRow(2);
            row.setHeightInPoints(30);
            for (int i = 0; i < 8; i++) {
                //column width is set in units of 1/256th of a character width
                sheet.setColumnWidth(i, 256 * 15);
            }

            createCell(wb, row, 0, HorizontalAlignment.CENTER, VerticalAlignment.BOTTOM);
            createCell(wb, row, 1, HorizontalAlignment.CENTER_SELECTION, VerticalAlignment.BOTTOM);
            createCell(wb, row, 2, HorizontalAlignment.FILL, VerticalAlignment.CENTER);
            createCell(wb, row, 3, HorizontalAlignment.GENERAL, VerticalAlignment.CENTER);
            createCell(wb, row, 4, HorizontalAlignment.JUSTIFY, VerticalAlignment.JUSTIFY);
            createCell(wb, row, 5, HorizontalAlignment.LEFT, VerticalAlignment.TOP);
            createCell(wb, row, 6, HorizontalAlignment.RIGHT, VerticalAlignment.TOP);

            //center text over B4, C4, D4
            row = sheet.createRow(3);
            centerAcrossSelection(wb, row, 1, 3, VerticalAlignment.CENTER);

            // Write the output to a file
            try (OutputStream fileOut = new FileOutputStream("xssf-align.xlsx")) {
                wb.write(fileOut);
            }
        }
    }


    /**
     * 建一个单元格并以某种方式对齐它。
     * 参数：
     * wb——工作簿
     * row – 在其中创建单元格的行
     * column – 创建单元格的列号
     * halign - 单元格的水平对齐方式
     */
    private static void createCell(XSSFWorkbook wb, XSSFRow row, int column,
                                   HorizontalAlignment halign, VerticalAlignment valign){
        final XSSFCreationHelper ch = wb.getCreationHelper();
        final XSSFCell cell = row.createCell(column);
        cell.setCellValue(ch.createRichTextString("Align It"));
        final XSSFCellStyle cellStyle = wb.createCellStyle();
        cellStyle.setAlignment(halign);
        cellStyle.setVerticalAlignment(valign);
        cell.setCellStyle(cellStyle);
    }


    /**
     * Center a text over multiple columns using ALIGN_CENTER_SELECTION 合并同类
     *
     * @param wb the workbook
     * @param row the row to create the cell in
     * @param start_column  the column number to create the cell in and where the selection starts
     * @param end_column    the column number where the selection ends
     * @param valign the horizontal alignment for the cell.
     */
    private static void centerAcrossSelection(XSSFWorkbook wb, XSSFRow row,
                                              int start_column, int end_column, VerticalAlignment valign) {
        final XSSFCreationHelper ch = wb.getCreationHelper();

        // Create cell style with ALIGN_CENTER_SELECTION
        XSSFCellStyle cellStyle = wb.createCellStyle();
        cellStyle.setAlignment(HorizontalAlignment.CENTER_SELECTION);
        cellStyle.setVerticalAlignment(valign);

        // Create cells over the selected area
        for (int i = start_column; i <= end_column; i++) {
            final XSSFCell cell = row.createCell(i);
            cell.setCellStyle(cellStyle);
        }

        // Set value to the first cell
        final XSSFCell cell = row.getCell(start_column);
        cell.setCellValue(ch.createRichTextString("Align it"));

        // Make the selection
        final CTRowImpl ctRow = (CTRowImpl) row.getCTRow();

        // Add object with format start_coll:end_coll. For example 1:3 will span from
        // cell 1 to cell 3, where the column index starts with 0
        //
        // You can add multiple spans for one row
        final String span = start_column + ":" + end_column;

        final ArrayList<Object> spanList = new ArrayList<>();
        spanList.add(span);

        //add spns to the row
        ctRow.setSpans(spanList);
    }
}
