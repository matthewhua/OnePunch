package com.gameserver.dao.impl;

import com.gameserver.dao.PlayerDAO;
import com.gameserver.entity.Player;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.Transaction;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Optional;

/**
 * 玩家 DAO 实现类（Hibernate ORM）
 * 
 * 设计说明：
 * 1. 由 Dagger2 注入 SessionFactory，确保使用同一个 Hibernate 会话工厂
 * 2. 所有方法都是无状态的，可以被多个 Actor 并发调用
 * 3. 每个数据库操作都创建独立的 Session，操作完毕后自动关闭
 * 
 * 线程安全保证：
 * - SessionFactory 是线程安全的
 * - 每个线程获取独立的 Session
 * - 不维护任何可变状态
 * - 多个玩家 Actor 可以并发调用，无竞态条件
 */
public class PlayerDAOImpl implements PlayerDAO {
    
    private static final Logger logger = LoggerFactory.getLogger(PlayerDAOImpl.class);
    
    private final SessionFactory sessionFactory;

    public PlayerDAOImpl(SessionFactory sessionFactory) {
        this.sessionFactory = sessionFactory;
    }

    @Override
    public Optional<Player> findById(long playerId) {
        try (Session session = sessionFactory.openSession()) {
            Player player = session.get(Player.class, playerId);
            return Optional.ofNullable(player);
        } catch (Exception e) {
            logger.error("[DAO] 查询玩家失败: playerId={}", playerId, e);
            return Optional.empty();
        }
    }

    @Override
    public Optional<Player> findByAccount(String account) {
        try (Session session = sessionFactory.openSession()) {
            Player player = session.createQuery(
                "FROM Player WHERE account = :account", Player.class)
                .setParameter("account", account)
                .uniqueResult();
            return Optional.ofNullable(player);
        } catch (Exception e) {
            logger.error("[DAO] 查询玩家失败: account={}", account, e);
            return Optional.empty();
        }
    }

    @Override
    public boolean save(Player player) {
        try (Session session = sessionFactory.openSession()) {
            Transaction tx = session.beginTransaction();
            try {
                session.saveOrUpdate(player);
                tx.commit();
                return true;
            } catch (Exception e) {
                tx.rollback();
                logger.error("[DAO] 保存玩家失败: playerId={}", player.getId(), e);
                return false;
            }
        }
    }

    @Override
    public boolean delete(long playerId) {
        try (Session session = sessionFactory.openSession()) {
            Transaction tx = session.beginTransaction();
            try {
                Player player = session.get(Player.class, playerId);
                if (player != null) {
                    session.delete(player);
                }
                tx.commit();
                return true;
            } catch (Exception e) {
                tx.rollback();
                logger.error("[DAO] 删除玩家失败: playerId={}", playerId, e);
                return false;
            }
        }
    }

    @Override
    public int addExperience(long playerId, int experience) {
        try (Session session = sessionFactory.openSession()) {
            Transaction tx = session.beginTransaction();
            try {
                Player player = session.get(Player.class, playerId);
                if (player != null) {
                    player.addExperience(experience);
                    session.update(player);
                    tx.commit();
                    return player.getLevel();
                }
                return 0;
            } catch (Exception e) {
                tx.rollback();
                logger.error("[DAO] 增加经验值失败: playerId={}", playerId, e);
                return 0;
            }
        }
    }
}
