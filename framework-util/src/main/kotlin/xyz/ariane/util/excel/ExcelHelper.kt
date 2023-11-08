package xyz.ariane.util.excel

import org.apache.poi.hssf.usermodel.HSSFWorkbook
import org.apache.poi.ss.usermodel.Cell
import org.apache.poi.ss.usermodel.CellType
import org.apache.poi.ss.usermodel.HorizontalAlignment
import java.io.ByteArrayOutputStream
import java.util.*

/**
 * 描述表格的列
 */
open class TabColumn(
    val title: String,
    val dataIndex: String
)

/**
 * 描述了xls表格的Sheet
 */
data class SheetInfo(
    val sheetName: String,
    val columns: List<TabColumn>,
    val records: List<Map<String, Any>>
)

/**
 * 使用poi生成xls表格
 */
fun generateTable(
    sheets: LinkedList<SheetInfo>
): ByteArray {
    val wb = HSSFWorkbook()

    for (sheetInfo in sheets) {
        val sheetName = sheetInfo.sheetName
        val columns = sheetInfo.columns
        val records = sheetInfo.records

        val sheet = wb.createSheet(sheetName)

        // 表头样式
        val headStyle = wb.createCellStyle()
        headStyle.setAlignment(HorizontalAlignment.CENTER)
        val headFont = wb.createFont()
        headFont.bold = true
        headStyle.setFont(headFont)

        // 设置表头
        val headRow = sheet.createRow(0)
        for ((i, column) in columns.withIndex()) {
            val cell = headRow.createCell(i)
            sheet.setColumnWidth(i, 4500)
            cell.setCellStyle(headStyle)
            cell.setCellValue(column.title)
        }

        // 设置表体
        for ((i, record) in records.withIndex()) {
            val nowRow = i + 1
            val row = sheet.createRow(nowRow)

            for ((j, column) in columns.withIndex()) {
                val cell = row.createCell(j)
                cell.setCellValue(record[column.dataIndex].toString())
            }
        }
    }

    val bos = ByteArrayOutputStream()
    wb.write(bos)
    wb.close()
    bos.close()

    return bos.toByteArray()
}

/**
 * 读取poi表格里的值
 */
fun fetchCellStringValue(cell: Cell?): String {
    if (cell == null) return ""
    cell.setCellType(CellType.STRING)
    return cell.stringCellValue
}