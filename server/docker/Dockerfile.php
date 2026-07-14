FROM php:8.2-fpm

# Instalar extensiones
RUN docker-php-ext-install pdo pdo_pgsql

# Instalar Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Configurar working directory
WORKDIR /var/www/html

# Instalar dependencias PHP
COPY composer.json ./
RUN composer install --no-dev --optimize-autoloader

# Copiar código
COPY . .

# Permisos
RUN chown -R www-data:www-data /var/www/html

EXPOSE 9000

CMD ["php-fpm"]