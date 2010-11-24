// Copyright 2010 Filip Kunc. All rights reserved.

var movableImage = new Image();
movableImage.src = "Images/movable.png";

function FPMovablePlatform(x, y, widthSegments, heightSegments)
{
    this.x = x;
    this.y = y;
    this.widthSegments = widthSegments;
    this.heightSegments = heightSegments;
    this.moveY = 0.0;
    this.isVisible = true;

    this.isPlatform = function()
    {
        return true;
    }
    
    this.isMovable = function()
    {
        return true;
    }
    
    this.rect = function()
    {
        return new FPRect(this.x, this.y, this.widthSegments * 32.0, this.heightSegments * 32.0);
    }
    
    this.move = function(offsetX, offsetY)
    {
        this.x += offsetX;
        this.y += offsetY;
    }
    
    this.update = function(game)
    {
        var player = game.player;
        var playerRect = player.rect();

        this.moveY -= deceleration;
        if (this.moveY < maxFallSpeed)
        	this.moveY = maxFallSpeed;

        var moveRect = FPRectWithMove(this.rect(), 0.0, -this.moveY);

        if (FPRectIntersectsRectWithTolerance(playerRect, moveRect))
    		this.moveY = 0.0;

    	this.y -= this.moveY;
    	this.collisionUpDown(game);
    }
    
    this.collisionLeftRight = function(game)
    {
        for (i in game.gameObjects)
        {
            var platform = game.gameObjects[i];
            if (platform != this && platform.isPlatform())
            {
                var intersection = FPRectIntersection(platform.rect(), this.rect())
                if (intersection.isEmptyWithTolerance())
                    continue;
                    
                if (platform.rect().left() > this.rect().left())
                    return true;
                
                if (platform.rect().right() < this.rect().right())
                    return true;
            }
        }
        
        return false;
    }
    
    this.collisionUpDown = function(game)
    {
        var isColliding = false;
        
        for (i in game.gameObjects)
        {
            var platform = game.gameObjects[i];
            if (platform != this && platform.isPlatform())
            {
                var intersection = FPRectIntersection(platform.rect(), this.rect())
                if (intersection.isEmptyWithTolerance())
                    continue;
                    
                if (platform.rect().bottom() < this.rect().bottom())
                {
                    if (this.moveY > 0.0)
                        this.moveY = 0.0;
                    
                    this.y += intersection.size.height;
                    isColliding = true;
                }
                else if (this.moveY < 0.0)
                {
                    if (platform.rect().top() > this.rect().bottom() - tolerance + this.moveY)
                    {
                        this.moveY = 0.0;
                        this.y -= intersection.size.height;
                        isColliding = true;
                    }   
                }
                else if (platform.rect().top() > this.rect().bottom() - tolerance + this.moveY)
                {
                    this.y -= intersection.size.height;
                    isColliding = true;
                }
            }
        }
        
        return isColliding;
    }
    
    this.draw = function(context)
    {
        for (iy = 0; iy < this.heightSegments; iy++)
        {
            for (ix = 0; ix < this.widthSegments; ix++)
            {
                context.drawImage(movableImage, this.x + ix * 32.0, this.y + iy * 32.0);
            }
        }
    }
}