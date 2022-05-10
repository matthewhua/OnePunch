package io.hibernate.orm.demo;

import io.hibernate.orm.demo.entity.Skill;
import io.hibernate.orm.demo.entity.Tool;
import io.hibernate.orm.demo.entity.User;

import java.sql.*;
import java.util.*;

/**
 * The Class BasicJdbcDemo.
 *
 * @author Matthew Hua
 */
public class BasicJdbcDemo {


    public static void main(String[] args)  {
        try {
            //initDb();

            Tool tool = new Tool();
            tool.setId(1);
            tool.setName("Hammer");
            insertTool(tool);
            List<Tool> tools = new ArrayList<>();

            Skill skill = new Skill();
            skill.setId(1);
            skill.setName("Hammering Things");
            insertSkill(skill);
            List<Skill> skills = new ArrayList<Skill>();
            skills.add(skill);

            User user = new User();
            user.setId(1);
            user.setName("Matthew Hua");
            user.setEmail("1229926359@qq.com");
            user.setPhone("123-456-7890");
            user.setTools(tools);
            user.setSkills(skills);

            insertUser(user);

            user = getUser(user.getId());
            assert user != null;
            System.out.println(user);
        }catch (Exception e) {
            e.printStackTrace();
        }
        System.exit(0);

    }


    /**
     * Inits the db.
     *
     * @throws SQLException the SQL exception
     * @link https://www.jianshu.com/p/2543c71a8e45 详细介绍
     */

    private static void initDb() throws SQLException{
        Connection conn = null;    //自动关闭
        PreparedStatement stmt = null;

        try {
            conn = connection();

            stmt = conn.prepareStatement("CREATE TABLE users(id INT PRIMARY KEY, name VARCHAR(255), "
                    + "email VARCHAR(255), phone VARCHAR(255))");
            stmt.executeUpdate();
            stmt.close();
            stmt = conn.prepareStatement("CREATE TABLE tools(id INT PRIMARY KEY, name VARCHAR(255))");
            stmt.executeUpdate();
            stmt.close();
            stmt = conn.prepareStatement("CREATE TABLE skills(id INT PRIMARY KEY, name VARCHAR(255))");
            stmt.executeUpdate();
            stmt.close();
            stmt = conn.prepareStatement("CREATE TABLE users_tools(userId INT, toolId INT, "
                    + "PRIMARY KEY(userId, toolId))");
            stmt.executeUpdate();
            stmt.close();
            stmt = conn.prepareStatement("CREATE TABLE users_skills(userId INT, skillId INT, "
                    + "PRIMARY KEY(userId, skillId))");
            stmt.executeUpdate();
        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            if (stmt != null) {
                stmt.close();
            }
            if (conn != null) {
                conn.close();
            }
        }
    }


    private static void insertUser(User user) throws SQLException{
        Connection conn = null;
        PreparedStatement stmt = null;

        try {
            conn = connection();
            stmt = conn.prepareStatement( "INSERT INTO users VALUES(?, ?, ?, ?)" );
            stmt.setInt(1, user.getId());
            stmt.setString(2, user.getName());
            stmt.setString(3, user.getEmail());
            stmt.setString(4, user.getPhone());
            stmt.executeUpdate();
            stmt.close();

            for (Tool tool : user.getTools()) {
                stmt = conn.prepareStatement( "INSERT INTO users_tools VALUES(?, ?)" );
                stmt.setInt(1, user.getId());
                stmt.setInt(1, tool.getId());
                stmt.executeUpdate();
                stmt.close();
            }

            for (Skill skill : user.getSkills()) {
                stmt = conn.prepareStatement( "INSERT INTO users_skills VALUES(?, ?)" );
                stmt.setInt(1, user.getId());
                stmt.setInt(2, skill.getId());
                stmt.executeUpdate();
            }

        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            if (stmt != null) {
                stmt.close();
            }
            if (conn != null) {
                conn.close();
            }
        }

    }

    /**
     * Insert tool.
     *
     * @param tool the tool
     * @throws SQLException the SQL exception
     */
    private static void insertTool(Tool tool) throws SQLException {
        Connection conn = null;
        PreparedStatement stmt = null;

        try {
            conn = connection();

            stmt = conn.prepareStatement( "INSERT INTO tools VALUES(?, ?)" );
            stmt.setInt(1, tool.getId());
            stmt.setString(2, tool.getName());
            stmt.executeUpdate();
        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            if (stmt != null) {
                stmt.close();
            }
            if (conn != null) {
                conn.close();
            }
        }
    }

    /**
     * Insert skill.
     *
     * @param skill the skill
     * @throws SQLException the SQL exception
     */

    private static void insertSkill(Skill skill) throws SQLException{
        Connection conn = null;
        PreparedStatement stmt = null;

        try {
            conn = connection();

            stmt = conn.prepareStatement("INSERT INTO skills VALUES(?, ?)");
            stmt.setInt(1, skill.getId());
            stmt.setString(2, skill.getName());
            stmt.executeUpdate();
        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            if (stmt != null) {
                stmt.close();
            }
            if (conn != null) {
                conn.close();
            }
        }
    }


    private static User getUser(int id) throws SQLException{
        Connection conn = null;
        PreparedStatement stmt = null;
        ResultSet rs = null;

        try {
            conn = connection();

            stmt = conn.prepareStatement( "SELECT id, name, email, phone FROM users WHERE id=?" );
            stmt.setInt(1, id);
            rs = stmt.executeQuery();
            rs.next();

            User user = new User();
            user.setId(rs.getInt(1));
            user.setName(rs.getString(2));
            user.setEmail(rs.getString(3));
            user.setPhone(rs.getString(4));

            rs.close();
            stmt.close();

            user.setTools(new ArrayList<>());
            user.setSkills(new ArrayList<>());

            stmt = conn.prepareStatement( "SELECT tools.id, tools.name FROM tools, users_tools "
                    + "WHERE users_tools.userId=? AND users_tools.toolId=tools.id" );
            stmt.setInt(1, id);
            rs = stmt.executeQuery();
            while (rs.next()) {
                Tool tool = new Tool();
                tool.setId(rs.getInt(1));
                tool.setName(rs.getString(2));
                user.getTools().add(tool);
            }
            rs.close();
            stmt.close();

            stmt = conn.prepareStatement( "SELECT skills.id, skills.name FROM skills, users_skills "
                    + "WHERE users_skills.userId=? AND users_skills.skillId=skills.id" );
            stmt.setInt(1, id);
            rs = stmt.executeQuery();
            while (rs.next()) {
                Skill skill = new Skill();
                skill.setId(rs.getInt(1));
                skill.setName(rs.getString(2));
                user.getSkills().add(skill);
            }

            return user;

        } catch (Exception e) {
            e.printStackTrace();
            return null;
        } finally {
            if (rs != null) {
                rs.close();
            }
            if (stmt != null) {
                stmt.close();
            }
            if (conn != null) {
                conn.close();
            }
        }

    }


    /**
     * Connection.
     *
     * @return the connection
     * @throws SQLException           the SQL exception
     * @throws ClassNotFoundException the class not found exception
     */

    private static Connection connection() throws SQLException, ClassNotFoundException {
        Properties properties = new Properties();
        properties.put("user", "root");
        properties.put("password", "123456");
        Class.forName("com.mysql.jdbc.Driver"); // 注册驱动
        Connection connection = DriverManager.getConnection("jdbc:mysql://42.192.147.140:3306/hiberTest",
                properties);
        connection.setAutoCommit(true);
        return connection;
    }


}
