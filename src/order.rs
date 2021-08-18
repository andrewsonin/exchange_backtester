use crate::types::{Direction, OrderID, Price, Size};

pub(crate) trait Order {
    fn get_order_id(&self) -> OrderID;
    fn get_order_size(&self) -> Size;
    fn mut_order_size(&mut self) -> &mut Size;
    fn get_order_direction(&self) -> Direction;
}

pub(crate) trait PricedOrder: Order {
    fn get_price(&self) -> Price;
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrder {
    order_id: OrderID,
    size: Size,
    direction: Direction,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrder {
    order_id: OrderID,
    size: Size,
    direction: Direction,
    price: Price,
}

impl MarketOrder {
    pub const fn new(order_id: OrderID, size: Size, direction: Direction) -> MarketOrder {
        MarketOrder { order_id, size, direction }
    }
}

impl LimitOrder {
    pub const fn new(order_id: OrderID, size: Size, direction: Direction, price: Price) -> LimitOrder {
        LimitOrder { order_id, size, direction, price }
    }
}

impl const Order for MarketOrder {
    fn get_order_id(&self) -> OrderID { self.order_id }
    fn get_order_size(&self) -> Size { self.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.size }
    fn get_order_direction(&self) -> Direction { self.direction }
}

impl const Order for LimitOrder {
    fn get_order_id(&self) -> OrderID { self.order_id }
    fn get_order_size(&self) -> Size { self.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.size }
    fn get_order_direction(&self) -> Direction { self.direction }
}

impl const PricedOrder for LimitOrder {
    fn get_price(&self) -> Price { self.price }
}