-- --------------------------------------------------------
-- Host:                         C:\dev\anikibot\bot2\bot.db
-- Server version:               3.34.0
-- Server OS:                    
-- HeidiSQL Version:             11.2.0.6213
-- --------------------------------------------------------

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET NAMES  */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

-- Dumping data for table bot.channel: -1 rows
/*!40000 ALTER TABLE "channel" DISABLE KEYS */;
INSERT INTO "channel" ("id", "name", "prefix", "joined") VALUES
	(1, 'compileraddict', 'xD', 1);
/*!40000 ALTER TABLE "channel" ENABLE KEYS */;

-- Dumping data for table bot.command: -1 rows
/*!40000 ALTER TABLE "command" DISABLE KEYS */;
INSERT INTO "command" ("id", "name", "code") VALUES
	(1, 'hello', 'return "hi!"');
/*!40000 ALTER TABLE "command" ENABLE KEYS */;

-- Dumping data for table bot._sqlx_migrations: -1 rows
/*!40000 ALTER TABLE "_sqlx_migrations" DISABLE KEYS */;
INSERT INTO "_sqlx_migrations" ("version", "description", "installed_on", "success", "checksum", "execution_time") VALUES
	(20210401173116, 'init', '2021-04-04 16:43:19', 1, _binary 0x043F6F3F3F446A2203461C3F3F13223F3F3F3F4C3F3F3F5F3F18523F3F3353773F3F46623F3F653F3F4D3F3F33, 10341500);
/*!40000 ALTER TABLE "_sqlx_migrations" ENABLE KEYS */;

/*!40101 SET SQL_MODE=IFNULL(@OLD_SQL_MODE, '') */;
/*!40014 SET FOREIGN_KEY_CHECKS=IFNULL(@OLD_FOREIGN_KEY_CHECKS, 1) */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40111 SET SQL_NOTES=IFNULL(@OLD_SQL_NOTES, 1) */;
