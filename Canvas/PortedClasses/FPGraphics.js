// Copyright 2010 Filip Kunc. All rights reserved.

const rectTolerance = 0.1;

function contains(arr, obj) 
{
    var i = arr.length;
    while (i--) 
    {
        if (arr[i] === obj) 
            return true;
    }
    return false;
}

function FPPoint(x, y)
{
    this.x = x;
    this.y = y;
}

function FPSize(width, height)
{
    this.width = width;
    this.height = height;
    
    this.isEmpty = function()
    {
        if (this.width <= 0.0 || this.height <= 0.0)
            return true;
        return false;
    }
    
    this.isEmptyWithTolerance = function()
    {
        if (this.width <= rectTolerance || this.height <= rectTolerance)
            return true;
        return false;
    }
}

function FPRect(x, y, width, height)
{
    this.origin = new FPPoint(x, y);
    this.size = new FPSize(width, height);
    
    this.left = function()
    {
        return this.origin.x;
    }
    
    this.top = function()
    {
        return this.origin.y;
    }
    
    this.right = function()
    {
        return this.origin.x + this.size.width;
    }
    
    this.bottom = function()
    {
        return this.origin.y + this.size.height;
    }
    
    this.isEmpty = function()
    {
        return this.size.isEmpty();
    }
    
    this.isEmptyWithTolerance = function()
    {
        return this.size.isEmptyWithTolerance();
    }
}

function FPRectIntersection(a, b)
{
    var x = Math.max(a.left(), b.left());
    var y = Math.max(a.top(), b.top());
    
    var width = Math.min(a.right(), b.right()) - x;
    var height = Math.min(a.bottom(), b.bottom()) - y;
    
    var intersection = new FPRect(x, y, width, height);
    
    if (intersection.isEmpty())
        return new FPRect(0.0, 0.0, 0.0, 0.0);
    return intersection;
}

function FPRectWithMove(rc, moveX, moveY)
{
    var rect = new FPRect(0.0, 0.0, 0.0, 0.0);
    rect.origin = rc.origin;
    rect.size = rc.size;

    if (moveX < 0.0)
    {
        rect.origin.x += moveX;
        rect.size.width -= moveX;
    }
    else
    {
        rect.size.width += moveX;
    }
    
    if (moveY < 0.0)
	{
		rect.origin.y += moveY;
		rect.size.height -= moveY;
	}
	else
	{
		rect.size.height += moveY;
	}

	return rect;
}

function FPRectIntersectsRectWithTolerance(a, b)
{
    var intersection = FPRectIntersection(a, b);
    if (intersection.isEmptyWithTolerance())
        return false;
    return true;
}

