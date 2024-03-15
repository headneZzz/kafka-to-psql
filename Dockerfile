# Используем образ rustlang/rust:nightly как базовый
FROM rustlang/rust:nightly as builder

# Устанавливаем компоненты для кросс-компиляции в Windows
RUN rustup target add x86_64-pc-windows-gnu

# Устанавливаем MinGW для поддержки компиляции под Windows
RUN apt-get update && \
    apt-get install -y mingw-w64

# Копируем исходные коды вашего проекта в Docker контейнер
COPY . /usr/src/myapp
WORKDIR /usr/src/myapp

# Компилируем ваше приложение для Windows
RUN cargo build --release --target x86_64-pc-windows-gnu

# Настраиваем вторую стадию сборки, чтобы уменьшить размер конечного образа
# И используем scratch для создания минимального образа
FROM scratch as runtime

# Копируем скомпилированный exe файл из предыдущей стадии
COPY --from=builder /usr/src/myapp/target/x86_64-pc-windows-gnu/release/your_application.exe .

# Указываем команду, которая будет выполнена при запуске контейнера
# Для scratch образа эта команда не будет работать, так как scratch пустой,
# но мы используем ее для документирования
CMD ["./your_application.exe"]